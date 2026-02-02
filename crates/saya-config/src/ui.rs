use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UiConfig {
    /// Maximum number of text lines to show in overlay
    pub max_text_lines: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl UiConfig {
    pub fn new() -> Self {
        let max_text_lines = std::env::var("UI_MAX_TEXT_LINES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        Self { max_text_lines }
    }
}
