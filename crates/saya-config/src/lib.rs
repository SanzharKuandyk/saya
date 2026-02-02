use std::env;

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

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub ui: UiConfig,
    pub ocr: OcrConfig,
    pub anki: AnkiConfig,
    pub dictionary: DictionaryConfig,

    pub watchdog_timeout_ms: u64,
    /// App main loop delta time
    pub delta_time: u64,
    pub timeout_seconds: i32,
    /// Listen to websocket, if false use clipboard watcher
    pub listen_to_ws: bool,
    /// WebSocket URL to connect to
    pub ws_url: String,
}

impl Config {
    pub fn new() -> Self {
        let watchdog_timeout_ms = env::var("WATCHDOG_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10000); // 10 seconds default

        let delta_time = env::var("DELTA_TIME_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100); // 100ms default

        let timeout_seconds = env::var("TIMEOUT_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30); // 30 seconds default

        let ws_url = env::var("WS_URL").unwrap_or_else(|_| "ws://localhost:8080".to_string());

        Config {
            network: NetworkConfig::new(),
            ui: UiConfig::new(),
            ocr: OcrConfig::new(),
            anki: AnkiConfig::new(),
            dictionary: DictionaryConfig::new(),

            watchdog_timeout_ms,
            delta_time,
            timeout_seconds,
            listen_to_ws: false,
            ws_url,
        }
    }
}
