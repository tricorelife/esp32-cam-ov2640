# Portable Camera Stack

`esp32-cam-ov2640` now contains a portable camera stack boundary, not only the
OV2640 register layer.

## Layers

```text
application
  storage, transport, model worker, UI
  implements FrameSink or consumes Frame directly

portable stack in this crate
  CameraStack
  Frame / CaptureInfo / FrameInfo
  JpegScanner / JpegAssembler
  FrameQueue
  CameraSensor / SensorControls / CameraCapture / FrameSink traits
  Ov2640 sensor implementation

chip or board crate
  XCLK
  reset / power-down pins
  DVP/CSI/DCMI/LCD_CAM/PIO capture
  DMA descriptors and frame buffers
  implements CameraCapture
```

## What "Complete" Means Here

The crate is complete at the portable Rust boundary:

- It can initialize and control an OV2640 sensor.
- It defines the capture contract for any chip.
- It defines frame metadata and fixed-capacity frame storage helpers.
- It can assemble JPEG frames from stream chunks.
- It can queue frame metadata for backpressure/latest-frame policies.
- It can run a sensor + capture backend through `CameraStack`.

It intentionally does not own concrete chip peripherals. A single crate cannot
portably own ESP32-S3 LCD_CAM, STM32 DCMI, RP2040 PIO, and Linux V4L2 types
without becoming a set of feature-gated HAL adapters. Those adapters should be
separate crates that implement `CameraCapture`.

## Minimal Capture Adapter

```rust
use esp32_cam_ov2640::{CameraCapture, CaptureConfig, CaptureInfo};

struct MyCapture;

impl CameraCapture for MyCapture {
    type Error = MyCaptureError;

    fn start(&mut self, config: CaptureConfig) -> Result<(), Self::Error> {
        // Configure hardware for config.size and config.format.
        todo!()
    }

    fn capture_into(&mut self, buffer: &mut [u8]) -> Result<CaptureInfo, Self::Error> {
        // Capture one complete frame into buffer.
        todo!()
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}
```

## Streaming DMA Chunks

If the hardware emits chunks instead of complete frames, keep the chunk loop in
the adapter and use `JpegAssembler`:

```rust
use esp32_cam_ov2640::JpegAssembler;

let mut assembler = JpegAssembler::new();

loop {
    let chunk = next_dma_chunk();
    if let Some(bytes) = assembler.push_chunk(chunk, &mut frame_buffer)? {
        // frame_buffer[..bytes] is a complete JPEG.
        break;
    }
}
```

## Next Adapter Priority

1. ESP32-S3 LCD_CAM/GDMA `CameraCapture` adapter using the current GOOUUU
   continuous stream code.
2. Linux host mock/V4L2 adapter for regression tests without flashing hardware.
3. STM32 DCMI/DMA adapter once a target board is selected.
