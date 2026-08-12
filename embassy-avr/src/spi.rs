//! SPI driver for AVR (placeholder).
//!
//! This is a minimal stub. The full async SPI adapter will be implemented
//! once the embedded-hal version conflicts are resolved.

#[cfg(any(feature = "device-atmega328p", feature = "arduino-uno"))]
pub use atmega_hal::spi::Spi;
