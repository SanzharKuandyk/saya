use saya_types::types::CaptureRegion;
use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

fn default_auto() -> bool {
    true
}

fn default_language() -> String {
    "ja".to_string()
}

fn default_border_ready_color() -> String {
    "#00ff88".to_string()
}

fn default_border_capturing_color() -> String {
    "#ff4444".to_string()
}

fn default_border_preparing_color() -> String {
    "#ffaa00".to_string()
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
    #[serde(default = "default_border_ready_color")]
    pub border_ready_color: String,
    #[serde(default = "default_border_capturing_color")]
    pub border_capturing_color: String,
    #[serde(default = "default_border_preparing_color")]
    pub border_preparing_color: String,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            auto: default_auto(),
            language: default_language(),
            capture_region: None,
            target_window: None,
            border_ready_color: default_border_ready_color(),
            border_capturing_color: default_border_capturing_color(),
            border_preparing_color: default_border_preparing_color(),
        }
    }
}
