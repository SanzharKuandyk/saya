use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::state::AppState;

pub async fn watcher_io(
    _state: Arc<AppState>,
    delta_time: Duration,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(delta_time);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                todo!("clipboard/websocket work");
            }
            _ = cancel.cancelled() => {
                tracing::info!("watcher stopping");
                return Ok(());
            }
        }
    }
}
