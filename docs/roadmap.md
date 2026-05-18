# Roadmap

This roadmap focuses on making `esp32-cam-ov2640` comparable to the sensor
portion of a mature C OV2640 driver while preserving a portable Rust boundary.

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

## 0.2.x - Shared Sensor Traits

Introduce or align with a small trait layer for camera sensors:

- `probe`.
- `reset`.
- `configure_output`.
- `set_control`.
- `current_output`.
- `sensor_id`.

This should support additional sensor crates without forcing users into a
single SoC or board crate.

## Separate Crates, Not This Crate

The following work is important but intentionally out of scope for
`esp32-cam-ov2640`:

- ESP32-S3 LCD_CAM/GDMA capture helper.
- RP2040 PIO/DMA capture helper.
- STM32 DCMI/DMA capture helper.
- Board pin-map crates.
- JPEG frame extraction, buffering, and streaming protocols.
- Browser UI, model workers, or cloud integration.

Those should build on top of the portable sensor crate rather than being added
to it.
