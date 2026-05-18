# Changelog

## 0.1.0 - 2026-05-18

- Initial extracted `no_std` OV2640 sensor register driver.
- Supports OV2640 PID/VER detection.
- Supports QVGA YUV422, QVGA RGB565, QVGA JPEG, and experimental VGA JPEG setup helpers.
- Supports typed sensor controls for brightness, contrast, saturation, special
  effects, white balance, exposure, gain, mirror/flip, color bar, and JPEG
  quality.
- Supports typed DSP toggles for raw gamma, lens correction, DCW, bad-pixel
  correction, and white-pixel correction.
- Keeps board-specific XCLK, DVP capture, DMA, networking, and UI code out of the crate.
- Documents the intended split between the portable OV2640 sensor layer and
  future board/SoC capture crates.
