use embedded_hal::{delay::DelayNs, i2c::I2c};

use crate::registers::{
    BANK_DSP, BANK_SEL, BANK_SENSOR, COM7, COM7_SRST, END, OV2640_DSP_BYPASS_ON,
    OV2640_QVGA_JPEG_CLOCKS, OV2640_QVGA_WINDOW, OV2640_QVGA_YUV422_CLOCKS, OV2640_SETTINGS_CIF,
    OV2640_SETTINGS_JPEG, OV2640_SETTINGS_RGB565, OV2640_SETTINGS_TO_CIF, OV2640_SETTINGS_TO_SVGA,
    OV2640_SETTINGS_YUV422, OV2640_VGA_JPEG_CLOCKS, OV2640_VGA_WINDOW, QS, REG_PID, REG_VER,
    R_DVP_SP,
};

pub const DEFAULT_ADDRESS: u8 = 0x30;
pub const EXPECTED_PID: u8 = 0x26;
pub const EXPECTED_VER: u8 = 0x42;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    I2c(E),
    UnexpectedId { pid: u8, ver: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectedSensor {
    pub pid: u8,
    pub ver: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Qvga320x240,
    Vga640x480,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Yuv422,
    Rgb565,
    Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelClock {
    /// Verified for QVGA YUV422/RGB565 capture.
    QvgaParallel,
    /// Verified for QVGA JPEG on ESP32-S3 LCD_CAM/GDMA.
    QvgaJpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegConfig {
    pub qs: u8,
    pub r_dvp_sp: u8,
}

impl JpegConfig {
    pub const fn new(qs: u8, r_dvp_sp: u8) -> Self {
        Self { qs, r_dvp_sp }
    }
}

impl Default for JpegConfig {
    fn default() -> Self {
        Self {
            qs: 0x0c,
            r_dvp_sp: 0x08,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputConfig {
    pub resolution: Resolution,
    pub format: ImageFormat,
    pub pixel_clock: PixelClock,
    pub jpeg: JpegConfig,
}

impl OutputConfig {
    pub const fn qvga_yuv422() -> Self {
        Self {
            resolution: Resolution::Qvga320x240,
            format: ImageFormat::Yuv422,
            pixel_clock: PixelClock::QvgaParallel,
            jpeg: JpegConfig::new(0x0c, 0x08),
        }
    }

    pub const fn qvga_rgb565() -> Self {
        Self {
            resolution: Resolution::Qvga320x240,
            format: ImageFormat::Rgb565,
            pixel_clock: PixelClock::QvgaParallel,
            jpeg: JpegConfig::new(0x0c, 0x08),
        }
    }

    pub const fn qvga_jpeg(jpeg: JpegConfig) -> Self {
        Self {
            resolution: Resolution::Qvga320x240,
            format: ImageFormat::Jpeg,
            pixel_clock: PixelClock::QvgaJpeg,
            jpeg,
        }
    }

    pub const fn vga_jpeg(jpeg: JpegConfig) -> Self {
        Self {
            resolution: Resolution::Vga640x480,
            format: ImageFormat::Jpeg,
            pixel_clock: PixelClock::QvgaJpeg,
            jpeg,
        }
    }
}

pub struct Ov2640<I2C> {
    i2c: I2C,
    address: u8,
    current_bank: Option<u8>,
}

impl<I2C> Ov2640<I2C> {
    pub const fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: DEFAULT_ADDRESS,
            current_bank: None,
        }
    }

    pub const fn with_address(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            current_bank: None,
        }
    }

    pub fn free(self) -> I2C {
        self.i2c
    }

    pub const fn address(&self) -> u8 {
        self.address
    }
}

impl<I2C> Ov2640<I2C>
where
    I2C: I2c,
{
    pub fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(Error::I2c)?;
        if reg == BANK_SEL {
            self.current_bank = Some(value);
        }
        Ok(())
    }

    pub fn read_reg(&mut self, reg: u8) -> Result<u8, Error<I2C::Error>> {
        let mut value = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut value)
            .map_err(Error::I2c)?;
        Ok(value[0])
    }

    pub fn select_bank(&mut self, bank: u8) -> Result<(), Error<I2C::Error>> {
        if self.current_bank != Some(bank) {
            self.write_reg(BANK_SEL, bank)?;
        }
        Ok(())
    }

    pub fn write_regs(&mut self, regs: &[(u8, u8)]) -> Result<u16, Error<I2C::Error>> {
        let mut written = 0u16;

        for &(reg, value) in regs {
            if (reg, value) == END {
                break;
            }

            if reg == BANK_SEL {
                if self.current_bank != Some(value) {
                    self.write_reg(BANK_SEL, value)?;
                    written = written.wrapping_add(1);
                }
            } else {
                self.write_reg(reg, value)?;
                written = written.wrapping_add(1);
            }
        }

        Ok(written)
    }

    pub fn detect(&mut self) -> Result<DetectedSensor, Error<I2C::Error>> {
        self.select_bank(BANK_SENSOR)?;
        Ok(DetectedSensor {
            pid: self.read_reg(REG_PID)?,
            ver: self.read_reg(REG_VER)?,
        })
    }

    pub fn verify_id(&mut self) -> Result<DetectedSensor, Error<I2C::Error>> {
        let detected = self.detect()?;
        if detected.pid == EXPECTED_PID && detected.ver == EXPECTED_VER {
            Ok(detected)
        } else {
            Err(Error::UnexpectedId {
                pid: detected.pid,
                ver: detected.ver,
            })
        }
    }

    pub fn reset<D>(&mut self, delay: &mut D) -> Result<(), Error<I2C::Error>>
    where
        D: DelayNs,
    {
        self.write_reg(COM7, COM7_SRST)?;
        delay.delay_ms(10);
        self.current_bank = None;
        Ok(())
    }

    pub fn init<D>(
        &mut self,
        delay: &mut D,
        config: OutputConfig,
    ) -> Result<DetectedSensor, Error<I2C::Error>>
    where
        D: DelayNs,
    {
        let detected = self.verify_id()?;
        self.reset(delay)?;
        self.write_regs(OV2640_SETTINGS_CIF)?;
        self.configure_output(config)?;
        delay.delay_ms(100);
        Ok(detected)
    }

    pub fn init_qvga_yuv422<D>(
        &mut self,
        delay: &mut D,
    ) -> Result<DetectedSensor, Error<I2C::Error>>
    where
        D: DelayNs,
    {
        self.init(delay, OutputConfig::qvga_yuv422())
    }

    pub fn init_qvga_rgb565<D>(
        &mut self,
        delay: &mut D,
    ) -> Result<DetectedSensor, Error<I2C::Error>>
    where
        D: DelayNs,
    {
        self.init(delay, OutputConfig::qvga_rgb565())
    }

    pub fn init_qvga_jpeg<D>(
        &mut self,
        delay: &mut D,
        jpeg: JpegConfig,
    ) -> Result<DetectedSensor, Error<I2C::Error>>
    where
        D: DelayNs,
    {
        self.init(delay, OutputConfig::qvga_jpeg(jpeg))
    }

    pub fn init_vga_jpeg<D>(
        &mut self,
        delay: &mut D,
        jpeg: JpegConfig,
    ) -> Result<DetectedSensor, Error<I2C::Error>>
    where
        D: DelayNs,
    {
        self.init(delay, OutputConfig::vga_jpeg(jpeg))
    }

    pub fn configure_output(&mut self, config: OutputConfig) -> Result<(), Error<I2C::Error>> {
        match (config.resolution, config.format, config.pixel_clock) {
            (Resolution::Qvga320x240, ImageFormat::Yuv422, PixelClock::QvgaParallel) => {
                self.write_regs(OV2640_DSP_BYPASS_ON)?;
                self.write_regs(OV2640_SETTINGS_TO_CIF)?;
                self.write_regs(OV2640_QVGA_WINDOW)?;
                self.write_regs(OV2640_QVGA_YUV422_CLOCKS)?;
                self.write_regs(OV2640_SETTINGS_YUV422)?;
            }
            (Resolution::Qvga320x240, ImageFormat::Rgb565, PixelClock::QvgaParallel) => {
                self.write_regs(OV2640_DSP_BYPASS_ON)?;
                self.write_regs(OV2640_SETTINGS_TO_CIF)?;
                self.write_regs(OV2640_QVGA_WINDOW)?;
                self.write_regs(OV2640_QVGA_YUV422_CLOCKS)?;
                self.write_regs(OV2640_SETTINGS_RGB565)?;
            }
            (Resolution::Qvga320x240, ImageFormat::Jpeg, PixelClock::QvgaJpeg) => {
                self.write_regs(OV2640_DSP_BYPASS_ON)?;
                self.write_regs(OV2640_SETTINGS_TO_CIF)?;
                self.write_regs(OV2640_QVGA_WINDOW)?;
                self.write_regs(OV2640_QVGA_JPEG_CLOCKS)?;
                self.write_regs(OV2640_SETTINGS_JPEG)?;
                self.select_bank(BANK_DSP)?;
                self.write_reg(R_DVP_SP, config.jpeg.r_dvp_sp)?;
                self.write_reg(QS, config.jpeg.qs)?;
            }
            (Resolution::Vga640x480, ImageFormat::Jpeg, PixelClock::QvgaJpeg) => {
                self.write_regs(OV2640_DSP_BYPASS_ON)?;
                self.write_regs(OV2640_SETTINGS_TO_SVGA)?;
                self.write_regs(OV2640_VGA_WINDOW)?;
                self.write_regs(OV2640_VGA_JPEG_CLOCKS)?;
                self.write_regs(OV2640_SETTINGS_JPEG)?;
                self.select_bank(BANK_DSP)?;
                self.write_reg(R_DVP_SP, config.jpeg.r_dvp_sp)?;
                self.write_reg(QS, config.jpeg.qs)?;
            }
            _ => unreachable!("OutputConfig constructors only create supported combinations"),
        }

        Ok(())
    }
}
