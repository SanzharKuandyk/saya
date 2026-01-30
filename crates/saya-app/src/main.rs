use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use saya_core::types::AppEvent;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util_watchdog::Watchdog;

pub mod events;
pub mod io;
pub mod state;
pub mod ui;

use events::event_loop;
use io::watcher_io;
use ui::ui_loop;

use self::state::AppState;

#[tokio::main]
async fn main() {
    let _watchdog = Watchdog::builder()
        .watchdog_timeout(Duration::from_secs(10))
        .build();

    let state = Arc::new(AppState::new());

    let shutdown = async {
        signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    };

    run(state, shutdown).await;
}

pub async fn run(state: Arc<AppState>, shutdown: impl Future<Output = ()>) {
    let cancel = CancellationToken::new();

    let (app_to_ui_tx, app_to_ui_rx) = kanal::bounded_async::<AppEvent>(1);
    let (ui_to_app_tx, ui_to_app_rx) = kanal::unbounded_async::<AppEvent>();

    let event_loop = spawn_with_cancel(
        "event_loop",
        cancel.clone(),
        event_loop(state.clone(), ui_to_app_rx, app_to_ui_tx),
    );

    let ui = spawn_with_cancel(
        "ui_loop",
        cancel.clone(),
        ui_loop(state.clone(), app_to_ui_rx, ui_to_app_tx),
    );

    let watcher = {
        let state = state.clone();
        let cancel_child = cancel.child_token();
        spawn_with_cancel("watcher", cancel.clone(), async move {
            let delta_time = {
                let cfg = state.config.read().await;
                Duration::from_millis(cfg.delta_time)
            };
            watcher_io(state, delta_time, cancel_child).await
        })
    };

    tokio::select! {
        _ = shutdown => tracing::info!("Shutdown requested"),
        _ = event_loop => tracing::warn!("event_loop exited"),
        _ = ui => tracing::warn!("ui_loop exited"),
        _ = watcher => tracing::warn!("watcher exited"),
    }

    cancel.cancel();
}

fn spawn_with_cancel<F>(
    name: &'static str,
    cancel: CancellationToken,
    fut: F,
) -> tokio::task::JoinHandle<()>
where
    F: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        tokio::select! {
            result = fut => {
                if let Err(e) = result {
                    tracing::error!("{name} error: {e}");
                }
            }
            _ = cancel.cancelled() => {
                tracing::info!("{name} cancelled");
            }
        }
    })
}
