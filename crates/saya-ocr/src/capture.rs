use anyhow::{Context, Result};
use xcap::{Monitor, Window};

#[derive(Debug, Clone, Copy)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct RawImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// List all available windows with their titles
pub fn list_windows() -> Result<Vec<(u32, String)>> {
    let windows = Window::all().context("Failed to enumerate windows")?;
    Ok(windows
        .into_iter()
        .filter(|w| !w.title().is_empty() && !w.is_minimized())
        .map(|w| (w.id(), w.title().to_string()))
        .collect())
}

/// Capture a specific window by ID
pub fn capture_window(window_id: u32) -> Result<Vec<u8>> {
    let windows = Window::all().context("Failed to enumerate windows")?;
    let window = windows
        .into_iter()
        .find(|w| w.id() == window_id)
        .context("Window not found")?;

    let image = window.capture_image().context("Failed to capture window")?;
    encode_png(&image)
}

/// Capture window by title (partial match)
pub fn capture_window_by_title(title: &str) -> Result<Vec<u8>> {
    let windows = Window::all().context("Failed to enumerate windows")?;
    let window = windows
        .into_iter()
        .find(|w| w.title().to_lowercase().contains(&title.to_lowercase()))
        .context(format!("No window matching '{}'", title))?;

    let image = window.capture_image().context("Failed to capture window")?;
    encode_png(&image)
}

/// Capture the entire primary monitor
pub fn capture_primary_screen() -> Result<Vec<u8>> {
    let monitors = Monitor::all().context("Failed to get monitors")?;
    let monitor = monitors.first().context("No monitor found")?;

    let image = monitor.capture_image().context("Failed to capture screen")?;
    encode_png(&image)
}

/// Capture primary screen as raw RGBA
pub fn capture_primary_screen_raw() -> Result<RawImage> {
    let monitors = Monitor::all().context("Failed to get monitors")?;
    let monitor = monitors.first().context("No monitor found")?;

    let image = monitor.capture_image().context("Failed to capture screen")?;
    Ok(RawImage {
        width: image.width(),
        height: image.height(),
        data: image.into_raw(),
    })
}

/// Capture a region of the screen
pub fn capture_screen_region(region: CaptureRegion) -> Result<Vec<u8>> {
    let monitors = Monitor::all().context("Failed to get monitors")?;

    let monitor = monitors
        .iter()
        .find(|m| {
            region.x >= m.x()
                && region.y >= m.y()
                && region.x + region.width as i32 <= m.x() + m.width() as i32
                && region.y + region.height as i32 <= m.y() + m.height() as i32
        })
        .or(monitors.first())
        .context("No monitor found")?;

    let image = monitor.capture_image().context("Failed to capture screen")?;

    // Crop to region using xcap's image (0.25)
    let cropped = xcap::image::imageops::crop_imm(
        &image,
        (region.x - monitor.x()) as u32,
        (region.y - monitor.y()) as u32,
        region.width,
        region.height,
    )
    .to_image();

    encode_png(&cropped)
}

fn encode_png(image: &xcap::image::RgbaImage) -> Result<Vec<u8>> {
    use xcap::image::ImageEncoder;
    let mut buffer = Vec::new();
    xcap::image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(image.as_raw(), image.width(), image.height(), xcap::image::ExtendedColorType::Rgba8)
        .context("Failed to encode PNG")?;
    Ok(buffer)
}
