use saya_types::types::CaptureRegion;
use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

fn default_auto() -> bool {
    false
}

fn default_language() -> String {
    "ja".to_string()
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct OcrConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_auto")]
    pub auto: bool,
    #[serde(default = "default_language")]
    pub language: String,
    pub capture_region: Option<CaptureRegion>,
    pub target_window: Option<String>,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            auto: default_auto(),
            language: default_language(),
            capture_region: None,
            target_window: None,
        }
    }
}
