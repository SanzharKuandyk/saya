use std::env;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DictionaryConfig {
    pub enabled: bool,
    pub additional_paths: Vec<String>,
}

impl DictionaryConfig {
    pub fn new() -> Self {
        let enabled = env::var("DICT_ENABLED")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(true);

        let additional_paths = env::var("DICT_PATHS")
            .ok()
            .map(|paths| {
                paths
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            enabled,
            additional_paths,
        }
    }
}

impl Default for DictionaryConfig {
    fn default() -> Self {
        Self::new()
    }
}
