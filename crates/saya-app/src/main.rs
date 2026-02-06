use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use saya_lang_japanese::JapaneseProcessor;
use tokio::signal;
use tokio_util_watchdog::Watchdog;
use tracing_subscriber::util::SubscriberInitExt;

pub mod controller;
pub mod events;
pub mod io;
pub mod ocr_context;
pub mod profile;
pub mod state;
pub mod status;
pub mod ui;

#[cfg(test)]
mod tests;

use controller::AppController;
use state::AppState;

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

    profile::init_user_config().expect("failed to load user config");
    let config = profile::load_user_profile("main").expect("failed to load user profile");
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

    // Initialize processor and translator
    let processor = {
        let config = state.config.read().await;
        if config.dictionary.enabled {
            JapaneseProcessor::with_additional_dicts(&config.dictionary.additional_paths)
        } else {
            tracing::warn!("Dictionary disabled, using empty processor");
            JapaneseProcessor::with_additional_dicts(&[])
        }
    };

    let translator = {
        let config = state.config.read().await;
        if config.translator.enabled && !config.translator.api_key.is_empty() {
            Some(saya_lang_japanese::JapaneseTranslator::new(
                config.translator.api_key.clone(),
                config.translator.api_url.clone(),
            ))
        } else {
            None
        }
    };

    let processor = Arc::new(processor);
    let translator = Arc::new(translator);

    // Use controller for centralized task management
    let controller = AppController::new(state);
    let mut tasks = controller.spawn_tasks(processor, translator);

    tokio::select! {
        _ = shutdown => {
            tracing::info!("Shutdown requested (Ctrl+C)");
            controller.shutdown();
        }
        Some(result) = tasks.join_next() => {
            match result {
                Ok(Ok(_)) => {
                    tracing::info!("Task exited normally (likely UI closed) - shutting down");
                    controller.shutdown();
                }
                Ok(Err(e)) => {
                    tracing::error!("Task failed: {e}");
                    controller.shutdown();
                }
                Err(e) => {
                    tracing::error!("Task panicked: {e}");
                    controller.shutdown();
                }
            }
        }
    }

    // Wait for remaining tasks
    while let Some(result) = tasks.join_next().await {
        if let Err(e) = result {
            tracing::error!("Task cleanup error: {e}");
        }
    }

    tracing::info!("Application shutdown complete");
}

// Deprecated - kept for reference, not used
#[allow(dead_code)]
fn _deprecated_spawn_with_cancel<F>(
    name: &'static str,
    cancel: tokio_util::sync::CancellationToken,
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
