# Roadmap

This roadmap focuses on making `esp32-cam-ov2640` comparable to the portable
portion of a mature C camera driver while preserving a Rust boundary that can be
used across chips and projects.

## 0.1.x - Portable Stack Boundary

Implemented:

- `CameraSensor` trait for sensor probe/reset/output setup.
- `SensorControls` trait and typed `SensorControl` enum.
- `CameraCapture` trait for chip-specific capture backends.
- `FrameSink` trait for transports/storage/model pipelines.
- `FrameSize`, `FrameFormat`, `CaptureConfig`, `CaptureInfo`, `FrameInfo`,
  `Frame`, and fixed-capacity `FrameSlot`.
- Incremental `JpegScanner` and `JpegAssembler`.
- Fixed-capacity `FrameQueue` metadata queue for latest-frame/backpressure
  policies.
- `CameraStack` helper that initializes a sensor, starts capture, captures
  frames, validates JPEG frames, and writes frames to a sink.
- `examples/stack_mock.rs` hardware-free integration example.

## 0.1.x - Sensor Driver Completeness

Add typed controls that map to OV2640 register writes:

- Brightness. Implemented.
- Contrast. Implemented.
- Saturation. Implemented.
- Special effects. Implemented.
- Automatic white balance enable/disable. Implemented.
- White balance mode. Implemented.
- Automatic exposure enable/disable. Implemented.
- Exposure compensation / AE level. Implemented.
- Manual exposure value. Implemented with the OV2640 0..=1200 range used by
  Espressif's driver.
- Automatic gain enable/disable. Implemented.
- Manual AGC gain. Implemented with the OV2640 0..=30 table used by
  Espressif's driver.
- Gain ceiling. Implemented.
- Raw gamma toggle. Implemented.
- Lens correction toggle. Implemented.
- Downsize/crop/window toggle. Implemented.
- Bad-pixel correction toggle. Implemented.
- White-pixel correction toggle. Implemented.
- Mirror and vertical flip. Implemented.
- Color bar / test pattern. Implemented.
- Sharpness and denoise remain unimplemented because Espressif's OV2640 C
  driver currently marks those hooks unsupported.

Add output coverage:

- QQVGA 160x120.
- QVGA 320x240.
- CIF 352x288.
- VGA 640x480.
- SVGA 800x600 where stable.
- UXGA 1600x1200 only after a capture stack can validate the frame path.

Add documentation:

- Register source notes for each control group. Implemented in API docs and
  `docs/driver-scope.md`.
- Per-mode expected width, height, format, and frame timing assumptions.
- Platform integration examples that do not depend on ESP32-specific types.

## 0.2.x - Hardware Capture Adapters

Add external adapter crates or examples that implement `CameraCapture`:

- ESP32-S3 LCD_CAM/GDMA adapter.
- STM32 DCMI/DMA adapter.
- RP2040 PIO/DMA adapter.
- Linux I2C + V4L2 adapter for desktop validation.

These should depend on this crate; this crate should not depend on their HAL
types.

## 0.3.x - Multi-Sensor Traits

Align `CameraSensor` and `SensorControls` with additional sensor crates:

- OV5640.
- OV3660.
- GC0308.
- Other SCCB/I2C DVP sensors.

## Separate Crates, Not This Crate

The following work is important but intentionally out of scope for
`esp32-cam-ov2640`:

- Concrete ESP32-S3 LCD_CAM/GDMA capture helper.
- Concrete RP2040 PIO/DMA capture helper.
- Concrete STM32 DCMI/DMA capture helper.
- Board pin-map crates.
- Streaming protocols.
- Browser UI, model workers, or cloud integration.

Those should build on top of the portable sensor crate rather than being added
to it.
