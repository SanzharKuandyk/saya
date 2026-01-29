use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::windows::named_pipe::ServerOptions;
use tokio::signal;
use tokio::time::Interval;

pub mod state;

use self::state::AppState;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());

    // Shutdown future (Ctrl+C)
    let shutdown = async {
        signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    };

    run(state, shutdown).await;
}

pub async fn run(state: Arc<AppState>, shutdown: impl Future<Output = ()>) {
    let delta_time = {
        let config = state.config.read().await;
        Duration::from_millis(config.delta_time)
    };

    let interval = tokio::time::interval(delta_time);

    let watcher_state = Arc::clone(&state);
    let watcher = tokio::spawn(watcher_io(watcher_state, interval));

    let server_state = Arc::clone(&state);
    let server = tokio::spawn({
        async move {
            if let Err(e) = event_loop(server_state).await {
                tracing::error!("event_loop exited: {e}");
            }
        }
    });

    tokio::select! {
        _ = shutdown => {
            tracing::info!("Shutdown requested");
        }
        result = watcher => {
            match result {
                Ok(_) => tracing::warn!("watcher task exited"),
                Err(e) => tracing::error!("watcher task panicked: {e}"),
            }
        }
        result = server => {
            match result {
                Ok(_) => tracing::warn!("server task exited"),
                Err(e) => tracing::error!("server task panicked: {e}"),
            }
        }
    }
}

/// Main event loop
async fn event_loop(state: Arc<AppState>) -> Result<(), anyhow::Error> {
    let pipe_path = {
        let config = state.config.read().await;
        config.network.windows_pipe_path()
    };

    loop {
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_path)?;

        server.connect().await?;
    }
}

/// Watcher for websocket or clipboard
async fn watcher_io(_state: Arc<AppState>, mut interval: Interval) {
    loop {
        interval.tick().await;
    }
}
