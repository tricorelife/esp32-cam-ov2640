use crate::{ImageFormat, OutputConfig, Resolution};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSize {
    pub width: u16,
    pub height: u16,
}

impl FrameSize {
    pub const QQVGA: Self = Self {
        width: 160,
        height: 120,
    };
    pub const QVGA: Self = Self {
        width: 320,
        height: 240,
    };
    pub const VGA: Self = Self {
        width: 640,
        height: 480,
    };

    pub const fn pixels(self) -> u32 {
        self.width as u32 * self.height as u32
    }
}

impl From<Resolution> for FrameSize {
    fn from(value: Resolution) -> Self {
        match value {
            Resolution::Qvga320x240 => Self::QVGA,
            Resolution::Vga640x480 => Self::VGA,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    Yuv422,
    Rgb565,
    Jpeg,
}

impl From<ImageFormat> for FrameFormat {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::Yuv422 => Self::Yuv422,
            ImageFormat::Rgb565 => Self::Rgb565,
            ImageFormat::Jpeg => Self::Jpeg,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureConfig {
    pub size: FrameSize,
    pub format: FrameFormat,
}

impl CaptureConfig {
    pub const fn new(size: FrameSize, format: FrameFormat) -> Self {
        Self { size, format }
    }
}

impl From<OutputConfig> for CaptureConfig {
    fn from(value: OutputConfig) -> Self {
        Self {
            size: FrameSize::from(value.resolution),
            format: FrameFormat::from(value.format),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureInfo {
    pub bytes: usize,
    pub sequence: u64,
    pub timestamp_us: Option<u64>,
}

impl CaptureInfo {
    pub const fn new(bytes: usize, sequence: u64) -> Self {
        Self {
            bytes,
            sequence,
            timestamp_us: None,
        }
    }

    pub const fn with_timestamp(bytes: usize, sequence: u64, timestamp_us: u64) -> Self {
        Self {
            bytes,
            sequence,
            timestamp_us: Some(timestamp_us),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameInfo {
    pub size: FrameSize,
    pub format: FrameFormat,
    pub bytes: usize,
    pub sequence: u64,
    pub timestamp_us: Option<u64>,
}

impl FrameInfo {
    pub const fn new(
        size: FrameSize,
        format: FrameFormat,
        bytes: usize,
        sequence: u64,
        timestamp_us: Option<u64>,
    ) -> Self {
        Self {
            size,
            format,
            bytes,
            sequence,
            timestamp_us,
        }
    }

    pub const fn from_capture(config: CaptureConfig, capture: CaptureInfo) -> Self {
        Self {
            size: config.size,
            format: config.format,
            bytes: capture.bytes,
            sequence: capture.sequence,
            timestamp_us: capture.timestamp_us,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame<'a> {
    pub data: &'a [u8],
    pub info: FrameInfo,
}

impl<'a> Frame<'a> {
    pub const fn new(data: &'a [u8], info: FrameInfo) -> Self {
        Self { data, info }
    }
}

pub struct FrameSlot<const N: usize> {
    data: [u8; N],
    info: Option<FrameInfo>,
}

impl<const N: usize> FrameSlot<N> {
    pub const fn new() -> Self {
        Self {
            data: [0; N],
            info: None,
        }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn store(&mut self, info: FrameInfo) {
        self.info = Some(info);
    }

    pub fn clear(&mut self) {
        self.info = None;
    }

    pub const fn info(&self) -> Option<FrameInfo> {
        self.info
    }

    pub fn frame(&self) -> Option<Frame<'_>> {
        let info = self.info?;
        if info.bytes <= N {
            Some(Frame {
                data: &self.data[..info.bytes],
                info,
            })
        } else {
            None
        }
    }
}

impl<const N: usize> Default for FrameSlot<N> {
    fn default() -> Self {
        Self::new()
    }
}
