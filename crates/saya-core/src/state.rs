use saya_config::Config;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct AppState {
    pub config: RwLock<Config>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }
}
