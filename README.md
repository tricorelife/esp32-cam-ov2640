# esp32-cam-ov2640

`no_std` OV2640 sensor driver plus portable embedded camera stack primitives.

This crate is the reusable camera layer extracted from the GOOUUU
ESP32-S3-CAM bring-up work. It keeps the OV2640 register driver portable and
adds generic frame, JPEG, queue, capture, sink, and pipeline APIs that other
MCU/HAL projects can implement around their own camera peripherals.

## Scope

This crate covers the portable parts of an embedded camera stack. Common C
camera drivers often bundle sensor registers, board pins, DMA, frame buffers,
JPEG parsing, and network transports into one component. This Rust crate keeps
the reusable logic in one `no_std` crate and connects chip-specific code through
small traits.

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
- Generic `CameraSensor`, `SensorControls`, `CameraCapture`, and `FrameSink`
  traits.
- Frame metadata and fixed-capacity frame slots for `no_std` applications.
- Incremental JPEG SOI/EOI scanner and JPEG assembler for DMA stream chunks.
- Fixed-capacity metadata queue for latest-frame/backpressure policies.
- `CameraStack` pipeline that wires a sensor and a capture backend together.

Not included:

- Concrete XCLK generation for a specific chip.
- Concrete ESP32-S3 LCD_CAM/GDMA, STM32 DCMI/DMA, RP2040 PIO/DMA, or Linux
  V4L2 capture implementation.
- Concrete WiFi, HTTP, RTSP, WebRTC, LiveKit, cloud upload, model worker, or UI
  implementation.

Those pieces are board/SoC/application concerns. They plug into this crate by
implementing `CameraCapture` or `FrameSink`.

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

## Full Stack Integration

For a complete camera pipeline, keep the sensor driver here and implement the
capture backend in the target project:

```rust
use esp32_cam_ov2640::{
    CameraCapture, CameraStack, CaptureConfig, CaptureInfo, JpegConfig,
    OutputConfig, Ov2640,
};

struct MyCapture {
    // DCMI/LCD_CAM/PIO/V4L2 state owned by the target project.
}

impl CameraCapture for MyCapture {
    type Error = MyCaptureError;

    fn start(&mut self, config: CaptureConfig) -> Result<(), Self::Error> {
        // Configure pins, DMA descriptors, EOF mode, and interrupts.
        todo!()
    }

    fn capture_into(&mut self, buffer: &mut [u8]) -> Result<CaptureInfo, Self::Error> {
        // Fill `buffer` with one full frame and return byte count/sequence.
        todo!()
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

let sensor = Ov2640::new(i2c);
let capture = MyCapture { /* ... */ };
let output = OutputConfig::qvga_jpeg(JpegConfig::default());
let mut camera = CameraStack::new(sensor, capture, output);

let detected = camera.start(&mut delay)?;
let frame = camera.capture_jpeg_frame(&mut frame_buffer)?;
camera.stop()?;
```

For streaming DMA chunks instead of one full blocking frame, use
`JpegAssembler`:

```rust
use esp32_cam_ov2640::JpegAssembler;

let mut assembler = JpegAssembler::new();

for chunk in dma_chunks {
    if let Some(bytes) = assembler.push_chunk(chunk, &mut frame_buffer)? {
        // frame_buffer[..bytes] is one complete JPEG frame.
    }
}
```

See `examples/stack_mock.rs` for a hardware-free compileable example.

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
- A `CameraCapture` implementation if using `CameraStack`.

## Status

This is now a portable camera stack crate with an OV2640 sensor implementation
and reusable stack primitives. It is not a chip HAL crate: ESP32-S3
LCD_CAM/GDMA, STM32 DCMI/DMA, RP2040 PIO/DMA, and Linux V4L2 adapters should be
implemented in separate crates or applications and connected through
`CameraCapture`.

The planned sensor-level API work is tracked in
[docs/roadmap.md](docs/roadmap.md). The first typed control set and portable
stack boundary are implemented; the next priority is chip-specific capture
adapters and broader frame-size validation.

## License

Apache-2.0.

The OV2640 register tables are translated from Espressif's Apache-2.0
`esp32-camera` OV2640 settings and register headers, so this crate is published
under Apache-2.0.
