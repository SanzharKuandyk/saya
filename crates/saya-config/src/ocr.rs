use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OcrConfig {
    /// Enable OCR feature
    pub enabled: bool,
    /// OCR language code (e.g., "ja", "en")
    pub language: String,
    /// Screen region to capture (x, y, width, height)
    pub capture_region: Option<CaptureRegion>,
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
        Self {
            enabled: true,
            language: "ja".to_string(),
            capture_region: None,
        }
    }
}
