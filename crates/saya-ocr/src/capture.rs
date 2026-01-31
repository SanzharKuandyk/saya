use anyhow::{Context, Result};
use image::ImageEncoder;
use screenshots::Screen;

#[derive(Debug, Clone, Copy)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Capture a region of the screen and return PNG bytes
pub fn capture_screen_region(region: CaptureRegion) -> Result<Vec<u8>> {
    let screens = Screen::all().context("Failed to get screens")?;

    // Find the screen containing the region
    let screen = screens
        .iter()
        .find(|s| {
            let display = s.display_info;
            region.x >= display.x
                && region.y >= display.y
                && region.x + region.width as i32 <= display.x + display.width as i32
                && region.y + region.height as i32 <= display.y + display.height as i32
        })
        .or_else(|| screens.first())
        .context("No screen found")?;

    // Capture the region
    let image = screen
        .capture_area(region.x, region.y, region.width, region.height)
        .context("Failed to capture screen region")?;

    // Convert to PNG bytes
    let mut buffer = Vec::new();

    image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ColorType::Rgba8,
        )
        .context("Failed to encode image as PNG")?;

    Ok(buffer)
}
