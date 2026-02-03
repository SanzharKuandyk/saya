use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DictionaryConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub additional_paths: Vec<String>,
}

impl Default for DictionaryConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            additional_paths: vec![],
        }
    }
}
