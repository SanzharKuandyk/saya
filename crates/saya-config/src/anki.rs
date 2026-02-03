use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

fn default_url() -> String {
    "http://localhost:8765".to_string()
}

fn default_deck() -> String {
    "Japanese".to_string()
}

fn default_model() -> String {
    "Basic".to_string()
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct AnkiConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_deck")]
    pub deck: String,
    #[serde(default = "default_model")]
    pub model: String,
}

impl Default for AnkiConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            url: default_url(),
            deck: default_deck(),
            model: default_model(),
        }
    }
}
