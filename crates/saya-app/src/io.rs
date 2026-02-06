use std::sync::Arc;
use std::time::Duration;

use kanal::AsyncSender;
use saya_types::{AppEvent, TextSource};
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

pub async fn watcher_io(
    state: Arc<AppState>,
    _delta_time: Duration,
    cancel: CancellationToken,
    event_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    tracing::info!("watcher_io started");

    // Signal backend ready after brief initialization delay
    let ready_tx = event_tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let _ = ready_tx.send(AppEvent::BackendReady).await;
        tracing::info!("Backend ready signal sent");
    });

    let (listen_to_ws, ocr_enabled, hotkey_poll_interval_ms) = {
        let config = state.config.read().await;
        (
            config.listen_to_ws,
            config.ocr.enabled,
            config.hotkey_poll_interval_ms,
        )
    };

    // Spawn OCR hotkey listener if enabled
    if ocr_enabled {
        let tx = event_tx.clone();
        let cancel_clone = cancel.clone();

        tokio::task::spawn_blocking(move || {
            tracing::info!(">>> [HOTKEY] Starting hotkey listener...");

            let hotkey_manager = match saya_ocr::HotkeyManager::new() {
                Ok(m) => m,
                Err(e) => {
                    tracing::error!(">>> [HOTKEY] Failed to create hotkey: {}", e);
                    return;
                }
            };

            tracing::info!(">>> [HOTKEY] Ctrl+Shift+J registered, polling...");

            loop {
                if cancel_clone.is_cancelled() {
                    break;
                }

                if hotkey_manager.poll() {
                    tracing::debug!(">>> [HOTKEY] Hotkey pressed!");

                    // Send simple event - let event loop handle it
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx_clone.send(AppEvent::HotkeyOcrTriggered).await;
                    });
                }

                std::thread::sleep(std::time::Duration::from_millis(hotkey_poll_interval_ms));
            }

            tracing::info!(">>> [HOTKEY] Listener stopped");
        });
    }

    if listen_to_ws {
        let ws_url = {
            let config = state.config.read().await;
            config.ws_url.clone()
        };

        saya_io::ws::start_ws_listener(&ws_url, move |text| {
            let tx = event_tx.clone();
            tokio::spawn(async move {
                let _ = tx
                    .send(AppEvent::RawTextInput {
                        text: text.clone(),
                        source: TextSource::Websocket,
                    })
                    .await;
                let _ = tx.send(AppEvent::TextInput(text)).await;
            });
        })
        .await?;

        cancel.cancelled().await;
    } else {
        let tx = event_tx.clone();
        tokio::select! {
            result = saya_io::clipboard::watch_clipboard(move |text| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(AppEvent::RawTextInput {
                        text: text.clone(),
                        source: TextSource::Clipboard,
                    }).await;
                    let _ = tx.send(AppEvent::TextInput(text)).await;
                });
            }) => {
                if let Err(e) = result {
                    tracing::error!("Clipboard watcher error: {}", e);
                }
            }
            _ = cancel.cancelled() => {}
        }
    }

    Ok(())
}
