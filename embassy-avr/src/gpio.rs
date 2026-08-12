//! GPIO pin wrappers for AVR.
//!
//! This module re-exports the [`avr_hal_generic::port`](avr_hal_generic::port) types
//! and crate-level `Peripheral` definitions so that `ariel-os-avr` can wire up
//! pins via the standard Embassy pattern.

pub use avr_hal_generic::port;

/// A marker trait for types that are owned by the application.
///
/// `Peri<T>` is a zero-cost wrapper that prevents the wrapped GPIO pin from
/// being moved into another context, enforcing singleton ownership at compile
/// time. This mirrors the pattern used by `embassy-nrf`, `embassy-rp`, and
/// `embassy-stm32`.
///
/// `Peri` is essentially the same as `avr-hal`'s `Pin` types but adds an
/// `embassy` re-export layer.
pub struct Peri<T>(T);

impl<T> Peri<T> {
    /// Wrap a raw pin in a `Peri` to take ownership.
    pub fn new(p: T) -> Self {
        Self(p)
    }

    /// Consume the wrapper and return the inner pin.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> core::ops::Deref for Peri<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> core::ops::DerefMut for Peri<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
