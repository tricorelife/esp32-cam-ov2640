# esp32-cam-ov2640

`no_std` OV2640 sensor register driver for Rust embedded projects.

This crate is the reusable sensor layer extracted from the GOOUUU ESP32-S3-CAM
bring-up work. It depends only on `embedded-hal` 1.0 for blocking I2C/SCCB and
delay traits.

## Scope

This crate intentionally covers the **sensor register layer** only. Common C
camera drivers often bundle the whole camera stack into one component; this
Rust crate keeps the portable OV2640 logic separate from board and SoC capture
code.

Included:

- OV2640 PID/VER detection.
- SCCB/I2C register read/write helpers.
- Espressif-derived OV2640 register tables translated to Rust.
- Verified QVGA 320x240 YUV422, RGB565, and JPEG configuration helpers.
- Experimental VGA 640x480 JPEG configuration helper.
- JPEG tuning parameters used by the current GOOUUU ESP32-S3-CAM baseline.
- Typed sensor controls for brightness, contrast, saturation, special effect,
  white balance, exposure, gain, DSP correction blocks, mirror/flip, color bar,
  and JPEG quality.

Not included:

- XCLK generation.
- ESP32-S3 LCD_CAM/GDMA capture.
- Frame buffers, JPEG extraction, or transport.
- WiFi, network transport, application model workers, or UI code.

Those pieces are board/SoC/application concerns and remain in the consuming
project.

See [docs/driver-scope.md](docs/driver-scope.md) for the mapping from common C
camera driver responsibilities to the Rust crate split used here.

## Current Verified Baseline

Hardware: GOOUUU ESP32-S3-CAM V1.3 with onboard OV2640.

Sensor parameters used by the current quality-first JPEG baseline:

```rust
use esp32_cam_ov2640::JpegConfig;

let jpeg = JpegConfig {
    qs: 0x0c,
    r_dvp_sp: 0x08,
};
```

The ESP32-S3 capture side should use frame-sync EOF for JPEG on this board:

```text
LCD_CAM EOF: VsyncSignal
STREAM_FRAME_SIZE=qvga
STREAM_FRAME_INTERVAL_MS=1
STREAM_OV2640_JPEG_QS=0x0c
STREAM_OV2640_R_DVP_SP=0x08
```

Avoid `ByteLen(2047)` / `STREAM_LCD_CAM_EOF_BYTES=2048` for the current JPEG
path. It can decode as syntactically valid JPEG while corrupting the lower part
of the image.

Experimental VGA/JPEG has also been verified as a complete 640x480 JPEG stream
with the same `QS=0x0c` and `R_DVP_SP=0x08` settings. On the current GOOUUU
board it runs around 2.2fps and is still limited by focus/orientation and sensor
capture wait time, so QVGA remains the conservative streaming baseline.

## Usage

Add the crate from crates.io after release:

```toml
[dependencies]
esp32-cam-ov2640 = "0.1"
```

Or use the Git repository directly:

```toml
[dependencies]
esp32-cam-ov2640 = { git = "https://github.com/tricorelife/esp32-cam-ov2640" }
```

Then configure the sensor after your board code has started XCLK and created an
I2C/SCCB peripheral:

```rust
use esp32_cam_ov2640::{JpegConfig, Ov2640};

let mut sensor = Ov2640::new(i2c);
let detected = sensor.init_qvga_jpeg(
    &mut delay,
    JpegConfig {
        qs: 0x0c,
        r_dvp_sp: 0x08,
    },
)?;

assert_eq!(detected.pid, 0x26);
assert_eq!(detected.ver, 0x42);
let i2c = sensor.free();
```

For experimental VGA/JPEG:

```rust
let detected = sensor.init_vga_jpeg(
    &mut delay,
    JpegConfig {
        qs: 0x0c,
        r_dvp_sp: 0x08,
    },
)?;
```

For uncompressed bring-up:

```rust
sensor.init_qvga_yuv422(&mut delay)?;
sensor.init_qvga_rgb565(&mut delay)?;
```

Sensor controls stay at the OV2640 register layer and can be used with any
capture backend. These methods mirror the sensor-register writes from
Espressif's OV2640 driver; the visible effect of each control should still be
validated on the target board and capture backend:

```rust
use esp32_cam_ov2640::{ControlLevel, GainCeiling, SpecialEffect, WhiteBalanceMode};

sensor.set_brightness(ControlLevel::Plus1)?;
sensor.set_contrast(ControlLevel::Zero)?;
sensor.set_saturation(ControlLevel::Plus2)?;
sensor.set_special_effect(SpecialEffect::None)?;
sensor.set_white_balance_mode(WhiteBalanceMode::Auto)?;
sensor.set_auto_white_balance(true)?;
sensor.set_auto_exposure(true)?;
sensor.set_exposure_level(ControlLevel::Zero)?;
sensor.set_auto_gain(true)?;
sensor.set_gain_ceiling(GainCeiling::X8)?;
sensor.set_raw_gamma(true)?;
sensor.set_lens_correction(true)?;
sensor.set_bad_pixel_correction(true)?;
sensor.set_white_pixel_correction(true)?;
sensor.set_downsize_crop_window(true)?;
sensor.set_horizontal_mirror(false)?;
sensor.set_vertical_flip(false)?;
sensor.set_color_bar(false)?;
sensor.set_jpeg_quality(0x0c)?;
```

## Integration Contract

The consuming board/application must provide:

- Stable OV2640 XCLK before detection. The verified GOOUUU setup uses 24MHz on
  GPIO15.
- SCCB/I2C at 7-bit address `0x30`.
- Correct DVP wiring and capture polarity for the target MCU.
- A capture mode that can receive a full frame. On ESP32-S3 JPEG, the verified
  mode is `EofMode::VsyncSignal`.

## Status

This is an extracted first crate boundary, not a complete camera stack. The next
crate boundary should move ESP32-S3 LCD_CAM/GDMA capture into a separate
`esp32-cam` or board-support crate after the current examples have been
converted to use this sensor API directly.

The planned sensor-level API work is tracked in
[docs/roadmap.md](docs/roadmap.md). The first typed control set is implemented;
the next priority is broader frame-size coverage and hardware validation of each
control across JPEG and uncompressed modes.

## License

Apache-2.0.

The OV2640 register tables are translated from Espressif's Apache-2.0
`esp32-camera` OV2640 settings and register headers, so this crate is published
under Apache-2.0.
