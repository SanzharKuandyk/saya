use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_config::Config;
use saya_types::AppEvent;
use tokio::sync::RwLock;

pub async fn ui_loop(
    app_to_ui_rx: AsyncReceiver<AppEvent>,
    ui_to_app_tx: AsyncSender<AppEvent>,
    config: Arc<RwLock<Config>>,
) -> anyhow::Result<()> {
    saya_ui::ui_loop(app_to_ui_rx, ui_to_app_tx, config).await
}
