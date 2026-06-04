use embedded_hal::delay::DelayNs;

use crate::{
    CaptureConfig, CaptureInfo, ControlLevel, GainCeiling, OutputConfig, SpecialEffect,
    WhiteBalanceMode,
};

pub trait CameraSensor {
    type Error;
    type Id;

    fn probe(&mut self) -> Result<Self::Id, Self::Error>;

    fn reset<D>(&mut self, delay: &mut D) -> Result<(), Self::Error>
    where
        D: DelayNs;

    fn configure_output(&mut self, config: OutputConfig) -> Result<(), Self::Error>;

    fn init<D>(&mut self, delay: &mut D, config: OutputConfig) -> Result<Self::Id, Self::Error>
    where
        D: DelayNs;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorControl {
    Brightness(ControlLevel),
    Contrast(ControlLevel),
    Saturation(ControlLevel),
    SpecialEffect(SpecialEffect),
    WhiteBalanceMode(WhiteBalanceMode),
    AutoWhiteBalance(bool),
    AwbGain(bool),
    AutoExposure(bool),
    Aec2(bool),
    ExposureLevel(ControlLevel),
    ExposureValue(u16),
    AutoGain(bool),
    AgcGain(u8),
    GainCeiling(GainCeiling),
    RawGamma(bool),
    LensCorrection(bool),
    DownsizeCropWindow(bool),
    BadPixelCorrection(bool),
    WhitePixelCorrection(bool),
    HorizontalMirror(bool),
    VerticalFlip(bool),
    ColorBar(bool),
    JpegQuality(u8),
}

pub trait SensorControls {
    type Error;

    fn set_control(&mut self, control: SensorControl) -> Result<(), Self::Error>;
}

pub trait CameraCapture {
    type Error;

    fn start(&mut self, config: CaptureConfig) -> Result<(), Self::Error>;

    fn capture_into(&mut self, buffer: &mut [u8]) -> Result<CaptureInfo, Self::Error>;

    fn stop(&mut self) -> Result<(), Self::Error>;
}

pub trait FrameSink {
    type Error;

    fn write_frame(&mut self, frame: crate::Frame<'_>) -> Result<(), Self::Error>;
}
