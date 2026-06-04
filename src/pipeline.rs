use embedded_hal::delay::DelayNs;

use crate::{
    is_complete_jpeg, CameraCapture, CameraSensor, CaptureConfig, Frame, FrameFormat, FrameInfo,
    FrameSink, OutputConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackError<SensorError, CaptureError, SinkError = core::convert::Infallible> {
    Sensor(SensorError),
    Capture(CaptureError),
    Sink(SinkError),
    BufferTooSmall { bytes: usize, capacity: usize },
    ExpectedJpeg,
    InvalidJpeg,
}

pub type StackResult<T, SensorError, CaptureError, SinkError = core::convert::Infallible> =
    Result<T, StackError<SensorError, CaptureError, SinkError>>;

pub struct CameraStack<SENSOR, CAPTURE> {
    sensor: SENSOR,
    capture: CAPTURE,
    output: OutputConfig,
    capture_config: CaptureConfig,
}

impl<SENSOR, CAPTURE> CameraStack<SENSOR, CAPTURE> {
    pub fn new(sensor: SENSOR, capture: CAPTURE, output: OutputConfig) -> Self {
        Self {
            sensor,
            capture,
            output,
            capture_config: CaptureConfig::from(output),
        }
    }

    pub fn split(self) -> (SENSOR, CAPTURE) {
        (self.sensor, self.capture)
    }

    pub const fn output(&self) -> OutputConfig {
        self.output
    }

    pub const fn capture_config(&self) -> CaptureConfig {
        self.capture_config
    }

    pub fn sensor_mut(&mut self) -> &mut SENSOR {
        &mut self.sensor
    }

    pub fn capture_mut(&mut self) -> &mut CAPTURE {
        &mut self.capture
    }
}

impl<SENSOR, CAPTURE> CameraStack<SENSOR, CAPTURE>
where
    SENSOR: CameraSensor,
    CAPTURE: CameraCapture,
{
    pub fn start<D>(
        &mut self,
        delay: &mut D,
    ) -> StackResult<SENSOR::Id, SENSOR::Error, CAPTURE::Error>
    where
        D: DelayNs,
    {
        let id = self
            .sensor
            .init(delay, self.output)
            .map_err(StackError::Sensor)?;
        self.capture
            .start(self.capture_config)
            .map_err(StackError::Capture)?;
        Ok(id)
    }

    pub fn capture_frame<'a>(
        &mut self,
        buffer: &'a mut [u8],
    ) -> StackResult<Frame<'a>, SENSOR::Error, CAPTURE::Error> {
        let capture = self
            .capture
            .capture_into(buffer)
            .map_err(StackError::Capture)?;
        if capture.bytes > buffer.len() {
            return Err(StackError::BufferTooSmall {
                bytes: capture.bytes,
                capacity: buffer.len(),
            });
        }

        let info = FrameInfo::from_capture(self.capture_config, capture);
        Ok(Frame::new(&buffer[..capture.bytes], info))
    }

    pub fn capture_jpeg_frame<'a>(
        &mut self,
        buffer: &'a mut [u8],
    ) -> StackResult<Frame<'a>, SENSOR::Error, CAPTURE::Error> {
        if self.capture_config.format != FrameFormat::Jpeg {
            return Err(StackError::ExpectedJpeg);
        }

        let frame = self.capture_frame(buffer)?;
        if !is_complete_jpeg(frame.data) {
            return Err(StackError::InvalidJpeg);
        }

        Ok(frame)
    }

    pub fn stop(&mut self) -> StackResult<(), SENSOR::Error, CAPTURE::Error> {
        self.capture.stop().map_err(StackError::Capture)
    }

    pub fn capture_and_write<SINK>(
        &mut self,
        buffer: &mut [u8],
        sink: &mut SINK,
    ) -> StackResult<(), SENSOR::Error, CAPTURE::Error, SINK::Error>
    where
        SINK: FrameSink,
    {
        let frame = self.capture_frame(buffer).map_err(|err| match err {
            StackError::Sensor(err) => StackError::Sensor(err),
            StackError::Capture(err) => StackError::Capture(err),
            StackError::BufferTooSmall { bytes, capacity } => {
                StackError::BufferTooSmall { bytes, capacity }
            }
            StackError::ExpectedJpeg => StackError::ExpectedJpeg,
            StackError::InvalidJpeg => StackError::InvalidJpeg,
            StackError::Sink(_) => unreachable!(),
        })?;

        sink.write_frame(frame).map_err(StackError::Sink)
    }
}
