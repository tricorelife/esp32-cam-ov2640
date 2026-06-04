use core::convert::Infallible;

use embedded_hal::delay::DelayNs;
use esp32_cam_ov2640::{
    CameraCapture, CameraSensor, CameraStack, CaptureConfig, CaptureInfo, JpegConfig, OutputConfig,
};

struct NoDelay;

impl DelayNs for NoDelay {
    fn delay_ns(&mut self, _ns: u32) {}
}

struct MockSensor {
    configured: bool,
}

impl CameraSensor for MockSensor {
    type Error = Infallible;
    type Id = u16;

    fn probe(&mut self) -> Result<Self::Id, Self::Error> {
        Ok(0x2640)
    }

    fn reset<D>(&mut self, _delay: &mut D) -> Result<(), Self::Error>
    where
        D: DelayNs,
    {
        self.configured = false;
        Ok(())
    }

    fn configure_output(&mut self, _config: OutputConfig) -> Result<(), Self::Error> {
        self.configured = true;
        Ok(())
    }

    fn init<D>(&mut self, delay: &mut D, config: OutputConfig) -> Result<Self::Id, Self::Error>
    where
        D: DelayNs,
    {
        let id = self.probe()?;
        self.reset(delay)?;
        self.configure_output(config)?;
        Ok(id)
    }
}

#[derive(Default)]
struct MockCapture {
    sequence: u64,
    started: bool,
}

impl CameraCapture for MockCapture {
    type Error = Infallible;

    fn start(&mut self, _config: CaptureConfig) -> Result<(), Self::Error> {
        self.started = true;
        Ok(())
    }

    fn capture_into(&mut self, buffer: &mut [u8]) -> Result<CaptureInfo, Self::Error> {
        assert!(self.started);
        let jpeg = [0xff, 0xd8, 0x01, 0x02, 0xff, 0xd9];
        buffer[..jpeg.len()].copy_from_slice(&jpeg);
        self.sequence += 1;
        Ok(CaptureInfo::new(jpeg.len(), self.sequence))
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        self.started = false;
        Ok(())
    }
}

fn main() {
    let sensor = MockSensor { configured: false };
    let capture = MockCapture::default();
    let output = OutputConfig::qvga_jpeg(JpegConfig::default());
    let mut stack = CameraStack::new(sensor, capture, output);
    let mut delay = NoDelay;
    let mut frame_buffer = [0; 64];

    let id = stack.start(&mut delay).unwrap();
    let frame = stack.capture_jpeg_frame(&mut frame_buffer).unwrap();
    stack.stop().unwrap();

    assert_eq!(id, 0x2640);
    assert_eq!(frame.info.sequence, 1);
    assert_eq!(frame.data, &[0xff, 0xd8, 0x01, 0x02, 0xff, 0xd9]);
}
