mod ocr;
mod capture;
mod hotkey;

pub use ocr::OcrEngine;
pub use capture::{capture_screen_region, CaptureRegion};
pub use hotkey::HotkeyManager;

use anyhow::Result;

/// Perform OCR on a screen region
pub async fn ocr_screen_region(region: CaptureRegion, language: &str) -> Result<String> {
    let image_data = capture_screen_region(region)?;
    let engine = OcrEngine::new(language)?;
    engine.recognize(&image_data).await
}
