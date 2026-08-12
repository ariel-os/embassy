//! Embassy Hardware Abstraction Layer (HAL) for AVR microcontrollers.
//!
//! `embassy-avr` wraps the [`avr-hal`](https://github.com/ariel-os/avr-hal) family
//! of crates (avr-hal-generic, atmega-hal, arduino-hal) to provide Embassy-style
//! async APIs for AVR microcontrollers.
//!
//! ## Supported devices
//!
//! - ATmega328P (Arduino Uno, Nano, Pro)
//!
//! ## Features
//!
//! - `device-atmega328p` — enable ATmega328P support via `atmega-hal`
//! - `arduino-uno` — enable Arduino Uno board crate (`arduino-hal`)
//! - `time-driver-tc0` — enable the [`time_driver::EmbassyAvrTimeDriver`]
//!   backed by Timer0 in CTC mode at ~1 kHz, registered as the global
//!   `embassy-time` driver.
//!
//! ## Limitations
//!
//! The ATmega328P has only 2 KB of SRAM and 32 KB of flash. As a result:
//!
//! - No `executor-interrupt` (only supported on Cortex-M); use `executor-thread`.
//! - No USB, BLE, networking, or hardware RNG.
//! - I2C/SPI drivers are blocking wrappers around the `nb` (non-blocking)
//!   `avr-hal` drivers — they spin until the hardware completes the transaction.
//!   This is acceptable for slow peripherals but means the executor does not
//!   yield during I2C/SPI transactions.
//! - UART RX is interrupt-driven; TX is polled.

#![no_std]
#![warn(missing_docs)]

pub mod gpio;
pub mod time_driver;
pub mod uart;
pub mod i2c;
pub mod spi;
pub mod storage;

pub use avr_device;
pub use avr_hal_generic as hal;

// Re-export at chip-level PAC and HAL for downstream `ariel-os-avr` consumers.
#[cfg(feature = "device-atmega328p")]
pub use atmega_hal as mcu;
#[cfg(feature = "arduino-uno")]
pub use arduino_hal as board;
