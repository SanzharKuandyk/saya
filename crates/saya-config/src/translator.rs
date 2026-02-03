use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    false
}

fn default_provider() -> String {
    "deepl".to_string()
}

fn default_from_lang() -> String {
    "ja".to_string()
}

fn default_to_lang() -> String {
    "en".to_string()
}

fn default_api_url() -> String {
    "https://api-free.deepl.com/v2/translate".to_string()
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TranslatorConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_from_lang")]
    pub from_lang: String,
    #[serde(default = "default_to_lang")]
    pub to_lang: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_api_url")]
    pub api_url: String,
}

impl Default for TranslatorConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            provider: default_provider(),
            from_lang: default_from_lang(),
            to_lang: default_to_lang(),
            api_key: String::new(),
            api_url: default_api_url(),
        }
    }
}
