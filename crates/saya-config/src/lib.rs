use serde::{Deserialize, Serialize};

use self::anki::AnkiConfig;
use self::dictionary::DictionaryConfig;
use self::network::NetworkConfig;
use self::ocr::OcrConfig;
use self::ui::UiConfig;

pub mod anki;
pub mod dictionary;
pub mod network;
pub mod ocr;
pub mod ui;

fn default_watchdog_timeout_ms() -> u64 {
    10000
}

fn default_delta_time() -> u64 {
    100
}

fn default_timeout_seconds() -> i32 {
    30
}

fn default_ws_url() -> String {
    "ws://localhost:8080".to_string()
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub network: NetworkConfig,
    pub ui: UiConfig,
    pub ocr: OcrConfig,
    pub anki: AnkiConfig,
    pub dictionary: DictionaryConfig,

    #[serde(default = "default_watchdog_timeout_ms")]
    pub watchdog_timeout_ms: u64,
    #[serde(default = "default_delta_time")]
    pub delta_time: u64,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: i32,
    #[serde(default)]
    pub listen_to_ws: bool,
    #[serde(default = "default_ws_url")]
    pub ws_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            ui: UiConfig::default(),
            ocr: OcrConfig::default(),
            anki: AnkiConfig::default(),
            dictionary: DictionaryConfig::default(),
            watchdog_timeout_ms: default_watchdog_timeout_ms(),
            delta_time: default_delta_time(),
            timeout_seconds: default_timeout_seconds(),
            listen_to_ws: false,
            ws_url: default_ws_url(),
        }
    }
}
