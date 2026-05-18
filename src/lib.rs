//! no_std OV2640 sensor register driver.
//!
//! This crate owns the sensor-level pieces: SCCB/I2C register access,
//! translated OV2640 register tables, sensor identification, reset, and the
//! currently verified QVGA output configurations. Board-specific XCLK,
//! parallel DVP capture, DMA, networking, and storage remain in the consumer
//! application or a separate SoC support crate.

#![no_std]

mod driver;
pub mod registers;

pub use driver::{
    ControlLevel, DetectedSensor, Error, GainCeiling, ImageFormat, JpegConfig, OutputConfig,
    Ov2640, PixelClock, Resolution, SpecialEffect, WhiteBalanceMode,
};
