use serde::{Deserialize, Serialize};
use translator::TranslatorConfig;

pub mod anki;
pub mod dictionary;
pub mod ocr;
pub mod translator;
pub mod ui;

use self::anki::AnkiConfig;
use self::dictionary::DictionaryConfig;
use self::ocr::OcrConfig;
use self::ui::UiConfig;

fn default_watchdog_timeout_ms() -> u64 {
    10000
}

fn default_hotkey_poll_interval_ms() -> u64 {
    50
}

fn default_auto_ocr_interval_ms() -> u64 {
    3000
}

fn default_watcher_interval_ms() -> u64 {
    100
}

fn default_timeout_seconds() -> i32 {
    30
}

fn default_ws_url() -> String {
    "ws://localhost:8080".to_string()
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub ui: UiConfig,
    pub ocr: OcrConfig,
    pub anki: AnkiConfig,
    pub dictionary: DictionaryConfig,
    pub translator: TranslatorConfig,

    #[serde(default = "default_watchdog_timeout_ms")]
    pub watchdog_timeout_ms: u64,
    #[serde(default = "default_hotkey_poll_interval_ms")]
    pub hotkey_poll_interval_ms: u64,
    #[serde(default = "default_auto_ocr_interval_ms")]
    pub auto_ocr_interval_ms: u64,
    #[serde(default = "default_watcher_interval_ms")]
    pub watcher_interval_ms: u64,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: i32,
    #[serde(default)]
    pub listen_to_ws: bool,
    #[serde(default = "default_ws_url")]
    pub ws_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_time: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            ocr: OcrConfig::default(),
            anki: AnkiConfig::default(),
            dictionary: DictionaryConfig::default(),
            translator: TranslatorConfig::default(),
            watchdog_timeout_ms: default_watchdog_timeout_ms(),
            hotkey_poll_interval_ms: default_hotkey_poll_interval_ms(),
            auto_ocr_interval_ms: default_auto_ocr_interval_ms(),
            watcher_interval_ms: default_watcher_interval_ms(),
            timeout_seconds: default_timeout_seconds(),
            listen_to_ws: false,
            ws_url: default_ws_url(),
            delta_time: None,
        }
    }
}
