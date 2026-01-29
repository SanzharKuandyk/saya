use std::env;

use serde::{Deserialize, Serialize};

/// TODO: Define proper purpose for NetworkConfig:
/// should it define interprocess configs or api call configs
/// or be combined(don't like this)
#[derive(Default, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Name of Windows pipe
    pub pipe_name: String,
}

impl NetworkConfig {
    pub fn new() -> Self {
        let pipe_name = env::var("WIN_PIPE_NAME").unwrap_or_else(|_| "saya-pipe".to_string());

        Self { pipe_name }
    }

    pub fn windows_pipe_path(&self) -> String {
        format!(r"\\.\pipe\{}", self.pipe_name)
    }
}
