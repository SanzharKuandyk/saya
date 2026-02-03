use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

fn default_language() -> String {
    "ja".to_string()
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct OcrConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_language")]
    pub language: String,
    pub capture_region: Option<CaptureRegion>,
    pub target_window: Option<String>,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            language: default_language(),
            capture_region: None,
            target_window: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
