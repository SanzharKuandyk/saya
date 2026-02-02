use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct AnkiConfig {
    /// Enable Anki integration
    pub enabled: bool,
    /// AnkiConnect URL
    pub url: String,
    /// Default deck name
    pub deck: String,
    /// Default model name
    pub model: String,
}

impl AnkiConfig {
    pub fn new() -> Self {
        Self {
            enabled: true,
            url: "http://localhost:8765".to_string(),
            deck: "Japanese".to_string(),
            model: "Basic".to_string(),
        }
    }
}
