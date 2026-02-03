use std::fs::File;
use std::future::Future;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;

use saya_config::Config;
use saya_core::types::AppEvent;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util_watchdog::Watchdog;
use tracing_subscriber::util::SubscriberInitExt;

pub mod events;
pub mod io;
pub mod state;
pub mod ui;

#[cfg(test)]
mod tests;

use events::event_loop;
use io::watcher_io;
use ui::ui_loop;

use self::state::AppState;

#[tokio::main(worker_threads = 4)]
async fn main() {
    // Initialize tracing subscriber for console logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .with_writer(std::io::stdout)
        .with_ansi(atty::is(atty::Stream::Stdout))
        .finish()
        .init();

    tracing::info!("Saya starting...");

    let config = load_config().expect("failed to load config...");

    let state = Arc::new(AppState::new(config));

    let watchdog_timeout = {
        let config = state.config.read().await;
        config.watchdog_timeout_ms
    };

    let _watchdog = Watchdog::builder()
        .watchdog_timeout(Duration::from_millis(watchdog_timeout))
        .build();

    let shutdown = async {
        signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    };

    run(state, shutdown).await;
}

pub async fn run(state: Arc<AppState>, shutdown: impl Future<Output = ()>) {
    tracing::info!("Application starting");
    let cancel = CancellationToken::new();

    let (app_to_ui_tx, app_to_ui_rx) = kanal::unbounded_async::<AppEvent>();
    let (ui_to_app_tx, ui_to_app_rx) = kanal::unbounded_async::<AppEvent>();

    let event_tx = ui_to_app_tx.clone();

    let event_loop = spawn_with_cancel(
        "event_loop",
        cancel.clone(),
        event_loop(state.clone(), ui_to_app_rx, app_to_ui_tx.clone()),
    );

    let ui = spawn_with_cancel(
        "ui_loop",
        cancel.clone(),
        ui_loop(state.clone(), app_to_ui_rx, event_tx.clone()),
    );

    let watcher = {
        let state = state.clone();
        let cancel_child = cancel.child_token();
        spawn_with_cancel("watcher", cancel.clone(), async move {
            let delta_time = {
                let cfg = state.config.read().await;
                Duration::from_millis(cfg.delta_time)
            };
            watcher_io(state, delta_time, cancel_child, app_to_ui_tx).await
        })
    };

    tokio::select! {
        _ = shutdown => {
            tracing::info!("Shutdown requested (Ctrl+C)");
            cancel.cancel();
        }
        _ = event_loop => {
            tracing::warn!("event_loop exited unexpectedly");
            cancel.cancel();
        }
        _ = ui => {
            tracing::warn!("UI exited - continuing without UI");
        }
        _ = watcher => {
            tracing::warn!("watcher exited - continuing without watcher");
        }
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

fn load_config() -> anyhow::Result<Config> {
    tracing::info!("Loading config...");

    let file = File::open("config.json")?;
    let reader = BufReader::new(file);

    let config: Config = serde_json::from_reader(reader)?;

    Ok(config)
}
