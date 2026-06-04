use embedded_hal::{delay::DelayNs, i2c::I2c};

use crate::registers::{
    AEC, AE_LEVEL_REGS, AGC_GAIN_TABLE, BANK_DSP, BANK_SEL, BANK_SENSOR, BRIGHTNESS_REGS, COM7,
    COM7_COLOR_BAR, COM7_SRST, COM8, COM8_AEC_EN, COM8_AGC_EN, COM9, CONTRAST_REGS, CTRL0, CTRL1,
    CTRL2, CTRL3, END, GAIN, OV2640_DSP_BYPASS_ON, OV2640_QVGA_JPEG_CLOCKS, OV2640_QVGA_WINDOW,
    OV2640_QVGA_YUV422_CLOCKS, OV2640_SETTINGS_CIF, OV2640_SETTINGS_JPEG, OV2640_SETTINGS_RGB565,
    OV2640_SETTINGS_TO_CIF, OV2640_SETTINGS_TO_SVGA, OV2640_SETTINGS_YUV422,
    OV2640_VGA_JPEG_CLOCKS, OV2640_VGA_WINDOW, QS, REG04, REG04_HFLIP_IMG, REG04_VFLIP_IMG,
    REG04_VREF_EN, REG45, REG_PID, REG_VER, R_DVP_SP, SATURATION_REGS, SPECIAL_EFFECT_REGS,
    WHITE_BALANCE_MODE_REGS,
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
pub enum ControlLevel {
    /// Minimum level used by Espressif's OV2640 tables.
    Minus2,
    /// Low level used by Espressif's OV2640 tables.
    Minus1,
    /// Neutral/default level.
    Zero,
    /// High level used by Espressif's OV2640 tables.
    Plus1,
    /// Maximum level used by Espressif's OV2640 tables.
    Plus2,
}

impl ControlLevel {
    const fn table_row(self) -> usize {
        match self {
            Self::Minus2 => 1,
            Self::Minus1 => 2,
            Self::Zero => 3,
            Self::Plus1 => 4,
            Self::Plus2 => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialEffect {
    /// Normal color output.
    None,
    /// Inverted colors.
    Negative,
    /// Black-and-white output.
    Grayscale,
    /// Red tint.
    Reddish,
    /// Green tint.
    Greenish,
    /// Blue tint.
    Blue,
    /// Sepia-like tint.
    Retro,
}

impl SpecialEffect {
    const fn table_row(self) -> usize {
        match self {
            Self::None => 1,
            Self::Negative => 2,
            Self::Grayscale => 3,
            Self::Reddish => 4,
            Self::Greenish => 5,
            Self::Blue => 6,
            Self::Retro => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteBalanceMode {
    /// Automatic white balance mode.
    Auto,
    /// Sunny/daylight preset.
    Sunny,
    /// Cloudy preset.
    Cloudy,
    /// Office/fluorescent preset.
    Office,
    /// Home/indoor preset.
    Home,
}

impl WhiteBalanceMode {
    const fn table_row(self) -> Option<usize> {
        match self {
            Self::Auto => None,
            Self::Sunny => Some(1),
            Self::Cloudy => Some(2),
            Self::Office => Some(3),
            Self::Home => Some(4),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GainCeiling {
    /// 2x automatic gain ceiling.
    X2,
    /// 4x automatic gain ceiling.
    X4,
    /// 8x automatic gain ceiling.
    X8,
    /// 16x automatic gain ceiling.
    X16,
    /// 32x automatic gain ceiling.
    X32,
    /// 64x automatic gain ceiling.
    X64,
    /// 128x automatic gain ceiling.
    X128,
}

impl GainCeiling {
    const fn bits(self) -> u8 {
        match self {
            Self::X2 => 0,
            Self::X4 => 1,
            Self::X8 => 2,
            Self::X16 => 3,
            Self::X32 => 4,
            Self::X64 => 5,
            Self::X128 => 6,
        }
    }
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

    pub fn write_reg_in_bank(
        &mut self,
        bank: u8,
        reg: u8,
        value: u8,
    ) -> Result<(), Error<I2C::Error>> {
        self.select_bank(bank)?;
        self.write_reg(reg, value)
    }

    pub fn read_reg_in_bank(&mut self, bank: u8, reg: u8) -> Result<u8, Error<I2C::Error>> {
        self.select_bank(bank)?;
        self.read_reg(reg)
    }

    pub fn set_reg_bits(
        &mut self,
        bank: u8,
        reg: u8,
        offset: u8,
        mask: u8,
        value: u8,
    ) -> Result<(), Error<I2C::Error>> {
        let current = self.read_reg_in_bank(bank, reg)?;
        let shifted_mask = mask << offset;
        let next = (current & !shifted_mask) | ((value & mask) << offset);
        self.write_reg_in_bank(bank, reg, next)
    }

    pub fn write_reg_flag(
        &mut self,
        bank: u8,
        reg: u8,
        mask: u8,
        enable: bool,
    ) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(bank, reg, 0, mask, if enable { mask } else { 0 })
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
        self.write_reg_in_bank(BANK_SENSOR, COM7, COM7_SRST)?;
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

    /// Set OV2640 brightness using the five-level Espressif table.
    pub fn set_brightness(&mut self, level: ControlLevel) -> Result<(), Error<I2C::Error>> {
        self.write_control_regs(BANK_DSP, &BRIGHTNESS_REGS, level.table_row())
    }

    /// Set OV2640 contrast using the five-level Espressif table.
    pub fn set_contrast(&mut self, level: ControlLevel) -> Result<(), Error<I2C::Error>> {
        self.write_control_regs(BANK_DSP, &CONTRAST_REGS, level.table_row())
    }

    /// Set OV2640 color saturation using the five-level Espressif table.
    pub fn set_saturation(&mut self, level: ControlLevel) -> Result<(), Error<I2C::Error>> {
        self.write_control_regs(BANK_DSP, &SATURATION_REGS, level.table_row())
    }

    /// Set one of the OV2640 special color effects.
    pub fn set_special_effect(&mut self, effect: SpecialEffect) -> Result<(), Error<I2C::Error>> {
        self.write_control_regs(BANK_DSP, &SPECIAL_EFFECT_REGS, effect.table_row())
    }

    /// Select automatic or preset white balance coefficients.
    pub fn set_white_balance_mode(
        &mut self,
        mode: WhiteBalanceMode,
    ) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, 0xc7, 6, 1, u8::from(mode.table_row().is_some()))?;
        if let Some(row) = mode.table_row() {
            self.write_control_regs(BANK_DSP, &WHITE_BALANCE_MODE_REGS, row)?;
        }
        Ok(())
    }

    /// Enable or disable the DSP automatic white balance block.
    pub fn set_auto_white_balance(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, CTRL1, 3, 1, u8::from(enable))
    }

    /// Enable or disable automatic white balance gain.
    pub fn set_awb_gain(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, CTRL1, 2, 1, u8::from(enable))
    }

    /// Enable or disable the DSP raw gamma block.
    pub fn set_raw_gamma(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, CTRL1, 5, 1, u8::from(enable))
    }

    /// Enable or disable DSP lens correction.
    pub fn set_lens_correction(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, CTRL1, 1, 1, u8::from(enable))
    }

    /// Enable or disable the DSP DCW block.
    pub fn set_downsize_crop_window(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, CTRL2, 5, 1, u8::from(enable))
    }

    /// Enable or disable DSP bad-pixel correction.
    pub fn set_bad_pixel_correction(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, CTRL3, 7, 1, u8::from(enable))
    }

    /// Enable or disable DSP white-pixel correction.
    pub fn set_white_pixel_correction(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, CTRL3, 6, 1, u8::from(enable))
    }

    /// Enable or disable automatic gain control.
    pub fn set_auto_gain(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.write_reg_flag(BANK_SENSOR, COM8, COM8_AGC_EN, enable)
    }

    /// Enable or disable automatic exposure control.
    pub fn set_auto_exposure(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.write_reg_flag(BANK_SENSOR, COM8, COM8_AEC_EN, enable)
    }

    /// Enable or disable the DSP AEC2 path.
    ///
    /// This follows Espressif's OV2640 driver, where the written bit is inverted.
    pub fn set_aec2(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_DSP, CTRL0, 6, 1, u8::from(!enable))
    }

    /// Set automatic exposure bias using the five-level Espressif table.
    pub fn set_exposure_level(&mut self, level: ControlLevel) -> Result<(), Error<I2C::Error>> {
        self.write_control_regs(BANK_SENSOR, &AE_LEVEL_REGS, level.table_row())
    }

    /// Set manual exposure register value.
    ///
    /// Values above 1200 are clamped to match Espressif's OV2640 driver range.
    pub fn set_exposure_value(&mut self, value: u16) -> Result<(), Error<I2C::Error>> {
        let value = value.min(1200);
        self.set_reg_bits(BANK_SENSOR, REG04, 0, 0x03, (value & 0x03) as u8)?;
        self.write_reg_in_bank(BANK_SENSOR, AEC, ((value >> 2) & 0xff) as u8)?;
        self.set_reg_bits(BANK_SENSOR, REG45, 0, 0x3f, (value >> 10) as u8)
    }

    /// Set manual AGC gain table index.
    ///
    /// Values above 30 are clamped to match Espressif's OV2640 driver range.
    pub fn set_agc_gain(&mut self, gain: u8) -> Result<(), Error<I2C::Error>> {
        let gain = gain.min(30) as usize;
        self.write_reg_in_bank(BANK_SENSOR, GAIN, AGC_GAIN_TABLE[gain])
    }

    /// Set automatic gain ceiling.
    pub fn set_gain_ceiling(&mut self, ceiling: GainCeiling) -> Result<(), Error<I2C::Error>> {
        self.set_reg_bits(BANK_SENSOR, COM9, 5, 0x07, ceiling.bits())
    }

    /// Enable or disable the sensor color-bar test pattern.
    pub fn set_color_bar(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.write_reg_flag(BANK_SENSOR, COM7, COM7_COLOR_BAR, enable)
    }

    /// Enable or disable horizontal mirror.
    pub fn set_horizontal_mirror(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.write_reg_flag(BANK_SENSOR, REG04, REG04_HFLIP_IMG, enable)
    }

    /// Enable or disable vertical flip.
    pub fn set_vertical_flip(&mut self, enable: bool) -> Result<(), Error<I2C::Error>> {
        self.write_reg_flag(BANK_SENSOR, REG04, REG04_VREF_EN, enable)?;
        self.write_reg_flag(BANK_SENSOR, REG04, REG04_VFLIP_IMG, enable)
    }

    /// Set JPEG quantization scale (`QS`) value.
    ///
    /// Values above 63 are clamped to the OV2640 range.
    pub fn set_jpeg_quality(&mut self, qs: u8) -> Result<(), Error<I2C::Error>> {
        self.write_reg_in_bank(BANK_DSP, QS, qs.min(63))
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

    fn write_control_regs<const N: usize>(
        &mut self,
        bank: u8,
        table: &[[u8; N]],
        row: usize,
    ) -> Result<(), Error<I2C::Error>> {
        for (&reg, &value) in table[0].iter().zip(table[row].iter()) {
            self.write_reg_in_bank(bank, reg, value)?;
        }
        Ok(())
    }
}

impl<I2C> crate::traits::CameraSensor for Ov2640<I2C>
where
    I2C: I2c,
{
    type Error = Error<I2C::Error>;
    type Id = DetectedSensor;

    fn probe(&mut self) -> Result<Self::Id, Self::Error> {
        self.verify_id()
    }

    fn reset<D>(&mut self, delay: &mut D) -> Result<(), Self::Error>
    where
        D: DelayNs,
    {
        Ov2640::reset(self, delay)
    }

    fn configure_output(&mut self, config: OutputConfig) -> Result<(), Self::Error> {
        Ov2640::configure_output(self, config)
    }

    fn init<D>(&mut self, delay: &mut D, config: OutputConfig) -> Result<Self::Id, Self::Error>
    where
        D: DelayNs,
    {
        Ov2640::init(self, delay, config)
    }
}

impl<I2C> crate::traits::SensorControls for Ov2640<I2C>
where
    I2C: I2c,
{
    type Error = Error<I2C::Error>;

    fn set_control(&mut self, control: crate::traits::SensorControl) -> Result<(), Self::Error> {
        use crate::traits::SensorControl;

        match control {
            SensorControl::Brightness(level) => self.set_brightness(level),
            SensorControl::Contrast(level) => self.set_contrast(level),
            SensorControl::Saturation(level) => self.set_saturation(level),
            SensorControl::SpecialEffect(effect) => self.set_special_effect(effect),
            SensorControl::WhiteBalanceMode(mode) => self.set_white_balance_mode(mode),
            SensorControl::AutoWhiteBalance(enable) => self.set_auto_white_balance(enable),
            SensorControl::AwbGain(enable) => self.set_awb_gain(enable),
            SensorControl::AutoExposure(enable) => self.set_auto_exposure(enable),
            SensorControl::Aec2(enable) => self.set_aec2(enable),
            SensorControl::ExposureLevel(level) => self.set_exposure_level(level),
            SensorControl::ExposureValue(value) => self.set_exposure_value(value),
            SensorControl::AutoGain(enable) => self.set_auto_gain(enable),
            SensorControl::AgcGain(gain) => self.set_agc_gain(gain),
            SensorControl::GainCeiling(ceiling) => self.set_gain_ceiling(ceiling),
            SensorControl::RawGamma(enable) => self.set_raw_gamma(enable),
            SensorControl::LensCorrection(enable) => self.set_lens_correction(enable),
            SensorControl::DownsizeCropWindow(enable) => self.set_downsize_crop_window(enable),
            SensorControl::BadPixelCorrection(enable) => self.set_bad_pixel_correction(enable),
            SensorControl::WhitePixelCorrection(enable) => self.set_white_pixel_correction(enable),
            SensorControl::HorizontalMirror(enable) => self.set_horizontal_mirror(enable),
            SensorControl::VerticalFlip(enable) => self.set_vertical_flip(enable),
            SensorControl::ColorBar(enable) => self.set_color_bar(enable),
            SensorControl::JpegQuality(qs) => self.set_jpeg_quality(qs),
        }
    }
}
