//! Time driver for AVR (Timer0 CTC at 1 kHz).
//!
//! Implements [`embassy_time_driver::Driver`] on the ATmega328P so that
//! [`embassy_time::Timer`] and friends work alongside the Embassy executor's
//! `integrated-timers` feature.
//!
//! ## Operation
//!
//! Timer0 CTC mode with `OCR0A = 249` and prescaler `/64`. At 16 MHz this
//! generates a 1 kHz (1 ms) tick. The `__vector_14` (TIMER0_COMPA) ISR
//! increments the epoch counter and checks the alarm deadline.
//!
//! The alarm is one-shot: the ISR fires the callback (which calls
//! `Pend::pend` to wake the executor), then sets `at = u64::MAX` so it
//! won't fire again until the executor calls `set_alarm` with a new
//! deadline.
//!
//! ## Critical section
//!
//! Shared state is protected by `critical_section::with(...)`. The impl
//! comes from `avr-device` defaults (`restore-state-u8`) — the application
//! MUST NOT declare `critical-section` with `features = ["restore-state-*"]`.

use core::cell::{Cell, UnsafeCell};

use embassy_time_driver::{time_driver_impl, AlarmHandle, Driver};

/// The single alarm ID (the executor needs only one alarm).
const ALARM_ID: u8 = 0;

/// Milliseconds since boot, incremented by the Timer0 ISR (1 kHz tick).
static EPOCH_MS: critical_section::Mutex<Cell<u64>> =
    critical_section::Mutex::new(Cell::new(0));

/// Alarm state, accessed under critical_section only (ISR + thread mode).
struct AlarmState {
    allocated: bool,
    callback: Option<fn(*mut ())>,
    ctx: *mut (),
    at: u64,
}

// Safety: `*mut ()` is only ever written/read under critical_section.
unsafe impl Send for AlarmState {}
unsafe impl Sync for AlarmState {}

static ALARM: critical_section::Mutex<UnsafeCell<AlarmState>> =
    critical_section::Mutex::new(UnsafeCell::new(AlarmState {
        allocated: false,
        callback: None,
        ctx: core::ptr::null_mut(),
        at: u64::MAX,
    }));

/// Embassy time driver for AVR (Timer0 CTC 1 kHz) — a zero-sized type.
pub struct EmbassyAvrTimeDriver;

impl Driver for EmbassyAvrTimeDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| EPOCH_MS.borrow(cs).get())
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        critical_section::with(|cs| {
            let alarm = unsafe { &mut *ALARM.borrow(cs).get() };
            if alarm.allocated {
                None
            } else {
                alarm.allocated = true;
                Some(unsafe { AlarmHandle::new(ALARM_ID) })
            }
        })
    }

    fn set_alarm_callback(&self, _alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let alarm = unsafe { &mut *ALARM.borrow(cs).get() };
            alarm.callback = Some(callback);
            alarm.ctx = ctx;
        });
    }

    fn set_alarm(&self, _alarm: AlarmHandle, timestamp: u64) -> bool {
        let ts = if timestamp == u64::MAX { u64::MAX } else { timestamp };
        critical_section::with(|cs| {
            let alarm = unsafe { &mut *ALARM.borrow(cs).get() };
            alarm.at = ts;
        });
        ts > self.now()
    }
}

time_driver_impl!(static DRIVER: EmbassyAvrTimeDriver = EmbassyAvrTimeDriver);

// ---------------------------------------------------------------------------
// ISR: TIMER0_COMPA (vector 14)
// ---------------------------------------------------------------------------

/// `__vector_14` = TIMER0_COMPA (datasheet ATmega328P, Table 12-1).
///
/// Increments the 1 kHz epoch counter and checks the alarm deadline.
/// When reached, fires the registered callback (one-shot).
#[unsafe(no_mangle)]
unsafe extern "C" fn __vector_14() {
    // Clear OCF0A (write-one-to-clear).
    unsafe { write8(TIFR0, 1 << 1) };

    let (_epoch, fire, ctx) = critical_section::with(|cs| {
        let epoch = EPOCH_MS.borrow(cs).get() + 1;
        EPOCH_MS.borrow(cs).set(epoch);

        let alarm = unsafe { &mut *ALARM.borrow(cs).get() };
        if alarm.at <= epoch {
            alarm.at = u64::MAX; // one-shot
            (epoch, alarm.callback, alarm.ctx)
        } else {
            (epoch, None, core::ptr::null_mut())
        }
    });

    if let Some(callback) = fire {
        callback(ctx);
    }
}

// ---------------------------------------------------------------------------
// Timer0 hardware init
// ---------------------------------------------------------------------------

/// ATmega328P I/O register addresses used by Timer0.
const TCCR0A: u16 = 0x44;
const TCCR0B: u16 = 0x45;
const TCNT0: u16 = 0x46;
const OCR0A: u16 = 0x47;
const TIMSK0: u16 = 0x6E;
const TIFR0: u16 = 0x35;

/// Start Timer0 in CTC mode at 1 kHz and enable the COMPA interrupt.
///
/// Call this once during hardware init, before enabling global interrupts.
pub fn init() {
    unsafe {
        // WGM01 = 1, WGM00 = 0 -> CTC mode (clear on compare match A).
        write8(TCCR0A, 1 << 1);
        // CS02:0 = 000 -> timer stopped while configuring.
        write8(TCCR0B, 0);
        write8(TCNT0, 0);
        write8(OCR0A, 249);
        // Clear any pending OCF0A flag, then enable OCIE0A.
        write8(TIFR0, 1 << 1);
        write8(TIMSK0, 1 << 1);
        // CS02:0 = 011 (prescaler /64) -> start timer. 16 MHz / 64 / 250 = 1 kHz.
        write8(TCCR0B, (1 << 1) | (1 << 0));
    }
}

unsafe fn write8(addr: u16, val: u8) {
    unsafe { core::ptr::write_volatile(addr as *mut u8, val) }
}
