# Changelog

## 0.1.0 - 2026-05-18

- Initial extracted `no_std` OV2640 sensor register driver.
- Supports OV2640 PID/VER detection.
- Supports QVGA YUV422, QVGA RGB565, QVGA JPEG, and experimental VGA JPEG setup helpers.
- Keeps board-specific XCLK, DVP capture, DMA, networking, and UI code out of the crate.
