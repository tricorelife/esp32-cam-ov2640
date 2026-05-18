# Driver Scope

This document defines what belongs in `esp32-cam-ov2640` and what should remain
in other crates or applications.

## Typical C Camera Driver Layout

Many C camera drivers are shipped as a single board-focused component. They
often include several layers at once:

| Layer | Typical C driver responsibility | Belongs in this crate? |
|---|---|---|
| Sensor registers | Register constants, initialization tables, PID/VER detection, reset | Yes |
| SCCB/I2C helpers | `read_reg`, `write_reg`, batch register writes | Yes |
| Output mode setup | Resolution, pixel format, JPEG enable, clock divisors | Yes |
| Image controls | Brightness, contrast, saturation, AWB, exposure, gain, mirror, flip, test pattern | Yes, as typed sensor controls |
| XCLK and power pins | XCLK PWM/LEDC/MCO/PIO, reset pin, power-down pin | No |
| Parallel capture | D0-D7, PCLK, VSYNC, HREF, DCMI, LCD_CAM, PIO, DMA | No |
| Frame buffers | Heap/PSRAM buffers, double buffering, frame queues | No |
| Frame parsing | JPEG SOI/EOI scanning, RGB/YUV conversion, crop/scale | Usually no; use capture/image crates |
| Board pin maps | GOOUUU, AI Thinker, custom board mappings | No |
| Transport | HTTP MJPEG, RTSP, TCP bridge, WebRTC, cloud upload | No |
| Application UI/model code | Viewer UI, model workers, data channels | No |

## Rust Split

The recommended Rust split is:

```text
esp32-cam-ov2640
  Portable OV2640 sensor register driver.
  Depends on embedded-hal I2C and delay traits only.

camera-sensor traits
  Future shared traits for sensor detection, output mode selection, and
  controls across OV2640, OV5640, OV3660, GC0308, and similar sensors.

esp32-s3-camera-hal or board support crate
  XCLK, GPIO pin map, LCD_CAM/GDMA, DMA buffers, and capture timing.

application crate
  Storage, transport, streaming, UI, model workers, and product logic.
```

This keeps the OV2640 driver reusable on ESP32-S3, STM32, RP2040, Linux I2C
adapters, and other Rust embedded targets. Each target still needs its own
parallel capture implementation to receive image data from the sensor.

## Public API Boundary

This crate should expose:

- `Ov2640<I2C>` sensor handle.
- Sensor identification and reset.
- Low-level register read/write for bring-up and debugging.
- Typed output modes such as QVGA YUV422, QVGA RGB565, QVGA JPEG, and VGA JPEG.
- Typed image controls for sensor-level behavior.

This crate should not expose:

- ESP32-specific peripheral types.
- DMA buffer ownership.
- Network sockets or streaming protocols.
- Application-specific frame queues.

If an API cannot be implemented with `embedded-hal` I2C plus delay traits, it
probably does not belong in this crate.
