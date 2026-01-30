use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_core::types::AppEvent;

use crate::state::AppState;

/// App's main loop
pub async fn event_loop(
    state: Arc<AppState>,
    ui_to_app_rx: AsyncReceiver<AppEvent>,
    _app_to_ui_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    loop {
        match ui_to_app_rx.recv().await {
            Ok(event) => {
                handle_events(state.clone(), event).await?;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

async fn handle_events(_state: Arc<AppState>, event: AppEvent) -> anyhow::Result<()> {
    match event {
        AppEvent::ConfigChanged => {}
        AppEvent::UiEvent(_event) => {}
        AppEvent::ApiRequest(_event) => {}
    }

    Ok(())
}
