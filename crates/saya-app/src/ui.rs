use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_core::types::AppEvent;

use crate::state::AppState;

pub async fn ui_loop(
    _state: Arc<AppState>,
    _app_to_ui_rx: AsyncReceiver<AppEvent>,
    _ui_to_app_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    todo!("ui implementation")
}
