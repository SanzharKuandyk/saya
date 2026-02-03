use kanal::{AsyncReceiver, AsyncSender};
use saya_core::types::AppEvent;

pub async fn ui_loop(
    app_to_ui_rx: AsyncReceiver<AppEvent>,
    ui_to_app_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    saya_ui::ui_loop(app_to_ui_rx, ui_to_app_tx).await
}
