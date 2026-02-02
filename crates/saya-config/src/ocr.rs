use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OcrConfig {
    /// Enable OCR feature
    pub enabled: bool,
    /// OCR language code (e.g., "ja", "en")
    pub language: String,
    /// Screen region to capture (x, y, width, height)
    pub capture_region: Option<CaptureRegion>,
    /// Target window title for capture (partial match)
    pub target_window: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl OcrConfig {
    pub fn new() -> Self {
        let language = std::env::var("OCR_LANG").unwrap_or_else(|_| "ja".to_string());

        Self {
            enabled: true,
            language,
            capture_region: None,
            target_window: None,
        }
    }
}
