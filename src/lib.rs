//! no_std OV2640 sensor register driver and portable camera stack primitives.
//!
//! This crate owns the portable pieces: SCCB/I2C register access, translated
//! OV2640 register tables, sensor identification, output configuration, typed
//! image controls, frame metadata, JPEG framing, backpressure metadata queues,
//! and a small capture pipeline. Board-specific XCLK, parallel DVP capture,
//! DMA peripherals, and networking transports are plugged in through traits.

#![no_std]

mod driver;
pub mod frame;
pub mod jpeg;
pub mod pipeline;
pub mod queue;
pub mod registers;
pub mod traits;

pub use driver::{
    ControlLevel, DetectedSensor, Error, GainCeiling, ImageFormat, JpegConfig, OutputConfig,
    Ov2640, PixelClock, Resolution, SpecialEffect, WhiteBalanceMode,
};
pub use frame::{CaptureConfig, CaptureInfo, Frame, FrameFormat, FrameInfo, FrameSize, FrameSlot};
pub use jpeg::{is_complete_jpeg, JpegAssembler, JpegError, JpegFrameBounds, JpegScanner};
pub use pipeline::{CameraStack, StackError};
pub use queue::{FrameQueue, QueueError};
pub use traits::{CameraCapture, CameraSensor, FrameSink, SensorControl, SensorControls};
