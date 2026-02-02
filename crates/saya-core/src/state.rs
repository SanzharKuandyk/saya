use saya_config::Config;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct AppState {
    pub config: RwLock<Config>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(Config::new()),
        }
    }
}
