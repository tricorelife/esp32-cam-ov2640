#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JpegError {
    BufferTooSmall { capacity: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegFrameBounds {
    pub start: usize,
    pub end: usize,
}

impl JpegFrameBounds {
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegScanner {
    position: usize,
    frame_start: Option<usize>,
    last: Option<u8>,
}

impl JpegScanner {
    pub const fn new() -> Self {
        Self {
            position: 0,
            frame_start: None,
            last: None,
        }
    }

    pub const fn position(&self) -> usize {
        self.position
    }

    pub const fn in_frame(&self) -> bool {
        self.frame_start.is_some()
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn push(&mut self, byte: u8) -> Option<JpegFrameBounds> {
        let current = self.position;
        let previous = self.last;
        self.position = self.position.saturating_add(1);
        self.last = Some(byte);

        if previous == Some(0xff) && byte == 0xd8 {
            self.frame_start = Some(current.saturating_sub(1));
            return None;
        }

        if previous == Some(0xff) && byte == 0xd9 {
            if let Some(start) = self.frame_start.take() {
                return Some(JpegFrameBounds {
                    start,
                    end: current + 1,
                });
            }
        }

        None
    }

    pub fn scan_chunk(&mut self, chunk: &[u8]) -> Option<JpegFrameBounds> {
        for &byte in chunk {
            if let Some(frame) = self.push(byte) {
                return Some(frame);
            }
        }
        None
    }
}

impl Default for JpegScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegAssembler {
    in_frame: bool,
    len: usize,
    last: Option<u8>,
}

impl JpegAssembler {
    pub const fn new() -> Self {
        Self {
            in_frame: false,
            len: 0,
            last: None,
        }
    }

    pub const fn in_frame(&self) -> bool {
        self.in_frame
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn push_chunk(
        &mut self,
        chunk: &[u8],
        output: &mut [u8],
    ) -> Result<Option<usize>, JpegError> {
        for &byte in chunk {
            if !self.in_frame {
                if self.last == Some(0xff) && byte == 0xd8 {
                    self.in_frame = true;
                    self.len = 0;
                    self.write_byte(output, 0xff)?;
                    self.write_byte(output, 0xd8)?;
                    self.last = Some(byte);
                    continue;
                }

                self.last = Some(byte);
                continue;
            }

            self.write_byte(output, byte)?;

            if self.last == Some(0xff) && byte == 0xd9 {
                let bytes = self.len;
                self.in_frame = false;
                self.len = 0;
                self.last = Some(byte);
                return Ok(Some(bytes));
            }

            self.last = Some(byte);
        }

        Ok(None)
    }

    fn write_byte(&mut self, output: &mut [u8], byte: u8) -> Result<(), JpegError> {
        if self.len >= output.len() {
            self.reset();
            return Err(JpegError::BufferTooSmall {
                capacity: output.len(),
            });
        }

        output[self.len] = byte;
        self.len += 1;
        Ok(())
    }
}

impl Default for JpegAssembler {
    fn default() -> Self {
        Self::new()
    }
}

pub const fn is_complete_jpeg(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    data[0] == 0xff
        && data[1] == 0xd8
        && data[data.len() - 2] == 0xff
        && data[data.len() - 1] == 0xd9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_handles_markers_across_chunks() {
        let mut scanner = JpegScanner::new();
        assert_eq!(scanner.scan_chunk(&[0x00, 0xff]), None);
        assert_eq!(scanner.scan_chunk(&[0xd8, 0x11, 0xff]), None);
        assert_eq!(
            scanner.scan_chunk(&[0xd9]),
            Some(JpegFrameBounds { start: 1, end: 6 })
        );
    }

    #[test]
    fn assembler_extracts_first_complete_jpeg() {
        let mut assembler = JpegAssembler::new();
        let mut out = [0; 8];
        assert_eq!(assembler.push_chunk(&[0x00, 0xff], &mut out), Ok(None));
        assert_eq!(
            assembler.push_chunk(&[0xd8, 0x01, 0x02, 0xff, 0xd9, 0xff], &mut out),
            Ok(Some(6))
        );
        assert_eq!(&out[..6], &[0xff, 0xd8, 0x01, 0x02, 0xff, 0xd9]);
        assert!(is_complete_jpeg(&out[..6]));
    }
}
