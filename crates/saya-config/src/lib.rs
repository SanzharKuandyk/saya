use std::env;

use serde::{Deserialize, Serialize};

use self::network::NetworkConfig;
use self::ui::UiConfig;

pub mod network;
pub mod ui;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    /// App main loop delta time
    pub delta_time: u64,
    pub timeout_seconds: i32,
    /// Listen to websocket, if false use clipboard watcher
    pub listen_to_ws: bool,

    pub network: NetworkConfig,
    pub ui: UiConfig,
}

impl Config {
    pub fn new() -> Self {
        let delta_time = env::var("DELTA_TIME_MS")
            .expect("Failed to load `DELTA_TIME_MS` environment variable.")
            .parse()
            .expect("Failed to parse `DELTA_TIME_MS` environment variable as usize.");

        let timeout_seconds = env::var("TIMEOUT_SECONDS")
            .expect("Failed to load `TIMEOUT_SECONDS` environment variable.")
            .parse()
            .expect("Failed to parse `TIMEOUT_SECONDS` environment variable.");

        Config {
            network: NetworkConfig::new(),
            ui: UiConfig::new(),

            delta_time,
            timeout_seconds,
            listen_to_ws: false,
        }
    }
}
