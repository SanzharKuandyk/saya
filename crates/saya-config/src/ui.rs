use serde::{Deserialize, Serialize};

fn default_max_text_lines() -> u32 {
    3
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UiConfig {
    #[serde(default = "default_max_text_lines")]
    pub max_text_lines: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            max_text_lines: default_max_text_lines(),
        }
    }
}
