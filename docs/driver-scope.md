# Driver Scope

This document defines what belongs in `esp32-cam-ov2640` and what should remain
in chip HAL crates or applications.

## Typical C Camera Driver Layout

Many C camera drivers are shipped as a single board-focused component. They
often include several layers at once:

| Layer | Typical C driver responsibility | Belongs in this crate? |
|---|---|---|
| Sensor registers | Register constants, initialization tables, PID/VER detection, reset | Yes |
| SCCB/I2C helpers | `read_reg`, `write_reg`, batch register writes | Yes |
| Output mode setup | Resolution, pixel format, JPEG enable, clock divisors | Yes |
| Image controls | Brightness, contrast, saturation, AWB, exposure, gain, DSP correction blocks, mirror, flip, test pattern | Yes, implemented as typed sensor controls |
| XCLK and power pins | XCLK PWM/LEDC/MCO/PIO, reset pin, power-down pin | No |
| Parallel capture | D0-D7, PCLK, VSYNC, HREF, DCMI, LCD_CAM, PIO, DMA | Trait only; concrete HAL code stays out |
| Frame buffers | Heap/PSRAM buffers, double buffering, frame queues | Portable metadata/slot helpers yes; allocator/PSRAM policy no |
| Frame parsing | JPEG SOI/EOI scanning, RGB/YUV conversion, crop/scale | JPEG framing yes; conversion/crop/scale no |
| Board pin maps | GOOUUU, AI Thinker, custom board mappings | No |
| Transport | HTTP MJPEG, RTSP, TCP bridge, WebRTC, cloud upload | Sink trait only; concrete protocol code stays out |
| Application UI/model code | Viewer UI, model workers, data channels | No |

## Rust Split

The recommended Rust split is:

```text
esp32-cam-ov2640
  Portable OV2640 sensor driver and generic camera stack primitives.
  Owns sensor registers, frame metadata, JPEG framing, queues, capture/sink
  traits, and a small CameraStack pipeline.

camera-sensor traits
  Future shared traits for sensor detection, output mode selection, and
  controls across OV2640, OV5640, OV3660, GC0308, and similar sensors.

esp32-s3-camera-hal or board support crate
  XCLK, GPIO pin map, LCD_CAM/GDMA, DMA buffers, capture timing, and a
  CameraCapture implementation.

application crate
  Storage, transport, streaming, UI, model workers, and product logic.
```

This keeps the OV2640 driver reusable on ESP32-S3, STM32, RP2040, Linux I2C
adapters, and other Rust embedded targets. Each target still needs its own
parallel capture implementation to receive image data from the sensor.

## C Driver Parity

The first Rust API set mirrors the OV2640 sensor-control functions from
Espressif's Apache-2.0 `esp32-camera` driver where the C implementation writes
real OV2640 registers:

| C driver function | Rust method |
|---|---|
| `set_brightness` | `set_brightness(ControlLevel)` |
| `set_contrast` | `set_contrast(ControlLevel)` |
| `set_saturation` | `set_saturation(ControlLevel)` |
| `set_special_effect` | `set_special_effect(SpecialEffect)` |
| `set_wb_mode` | `set_white_balance_mode(WhiteBalanceMode)` |
| `set_whitebal` | `set_auto_white_balance(bool)` |
| `set_awb_gain` | `set_awb_gain(bool)` |
| `set_exposure_ctrl` | `set_auto_exposure(bool)` |
| `set_aec2` | `set_aec2(bool)` |
| `set_ae_level` | `set_exposure_level(ControlLevel)` |
| `set_aec_value` | `set_exposure_value(u16)` |
| `set_gain_ctrl` | `set_auto_gain(bool)` |
| `set_agc_gain` | `set_agc_gain(u8)` |
| `set_gainceiling` | `set_gain_ceiling(GainCeiling)` |
| `set_raw_gma` | `set_raw_gamma(bool)` |
| `set_lenc` | `set_lens_correction(bool)` |
| `set_dcw` | `set_downsize_crop_window(bool)` |
| `set_bpc` | `set_bad_pixel_correction(bool)` |
| `set_wpc` | `set_white_pixel_correction(bool)` |
| `set_hmirror` | `set_horizontal_mirror(bool)` |
| `set_vflip` | `set_vertical_flip(bool)` |
| `set_colorbar` | `set_color_bar(bool)` |
| `set_quality` | `set_jpeg_quality(u8)` |

Espressif's OV2640 `set_sharpness` and `set_denoise` hooks currently return
unsupported in that C driver, so this crate does not expose typed methods for
them yet. They should only be added after the underlying register behavior is
validated.

## Public API Boundary

This crate should expose:

- `Ov2640<I2C>` sensor handle.
- Sensor identification and reset.
- Low-level register read/write for bring-up and debugging.
- Typed output modes such as QVGA YUV422, QVGA RGB565, QVGA JPEG, and VGA JPEG.
- Typed image controls for sensor-level behavior. The first API set includes
  brightness, contrast, saturation, special effects, white balance, exposure,
  gain, raw gamma, lens correction, DCW, bad/white pixel correction,
  mirror/flip, color bar, and JPEG quality.
- Generic `CameraSensor`, `SensorControls`, `CameraCapture`, and `FrameSink`
  traits.
- `FrameInfo`, `FrameSlot`, `FrameQueue`, `JpegScanner`, `JpegAssembler`, and
  `CameraStack`.

This crate should not expose:

- ESP32-specific peripheral types.
- DMA peripheral ownership or PSRAM allocator policy.
- Network sockets or concrete streaming protocols.
- Application-specific frame queues.

If an API needs a concrete peripheral type, board pin map, operating system, or
network protocol, it probably does not belong in this crate. If it can be
expressed as a small `no_std` trait or reusable frame/JPEG primitive, it can
belong here.
