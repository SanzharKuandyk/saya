use std::sync::Arc;
use std::time::Duration;

use kanal::AsyncSender;
use saya_core::types::AppEvent;
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

pub async fn watcher_io(
    state: Arc<AppState>,
    _delta_time: Duration,
    cancel: CancellationToken,
    event_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    // Check config
    let (listen_to_ws, ocr_enabled, ocr_language, ocr_region) = {
        let config = state.config.read().await;
        (
            config.listen_to_ws,
            config.ocr.enabled,
            config.ocr.language.clone(),
            config.ocr.capture_region,
        )
    };

    // Spawn OCR hotkey listener if enabled
    if ocr_enabled {
        let tx = event_tx.clone();
        let cancel_clone = cancel.clone();

        tokio::task::spawn_blocking(move || {
            let hotkey_manager = saya_ocr::HotkeyManager::new();
            if let Err(e) = &hotkey_manager {
                tracing::error!("Failed to create OCR hotkey manager: {}", e);
                return;
            }
            let hotkey_manager = hotkey_manager.unwrap();

            tracing::info!("OCR hotkey registered (Ctrl+Shift+S)");

            loop {
                if cancel_clone.is_cancelled() {
                    break;
                }

                // Poll for hotkey press
                if hotkey_manager.poll() {
                    tracing::info!("OCR hotkey pressed");

                    // Capture screen region and perform OCR
                    let region = if let Some(cfg_region) = ocr_region {
                        saya_ocr::CaptureRegion {
                            x: cfg_region.x,
                            y: cfg_region.y,
                            width: cfg_region.width,
                            height: cfg_region.height,
                        }
                    } else {
                        saya_ocr::CaptureRegion {
                            x: 0,
                            y: 0,
                            width: 800,
                            height: 600,
                        }
                    };

                    match saya_ocr::capture_screen_region(region) {
                        Ok(image_data) => {
                            let lang = ocr_language.clone();
                            let tx = tx.clone();

                            tokio::spawn(async move {
                                match saya_ocr::OcrEngine::new(&lang) {
                                    Ok(engine) => match engine.recognize(&image_data).await {
                                        Ok(text) => {
                                            if !text.trim().is_empty() {
                                                tracing::info!("OCR result: {}", text);
                                                if let Err(e) = tx.send(AppEvent::TextInput(text)).await {
                                                    tracing::error!("Failed to send OCR text: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("OCR recognition failed: {}", e)
                                        }
                                    },
                                    Err(e) => tracing::error!("Failed to create OCR engine: {}", e),
                                }
                            });
                        }
                        Err(e) => tracing::error!("Failed to capture screen: {}", e),
                    }
                }

                // Sleep briefly to avoid busy loop
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            tracing::info!("OCR hotkey listener stopping");
        });
    }

    if listen_to_ws {
        // WebSocket listener
        let ws_url = {
            let config = state.config.read().await;
            config.ws_url.clone()
        };

        tracing::info!("Starting WebSocket listener on {}", ws_url);

        saya_io::ws::start_ws_listener(&ws_url, move |text| {
            let tx = event_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = tx.send(AppEvent::TextInput(text)).await {
                    tracing::error!("Failed to send WebSocket text to app: {}", e);
                }
            });
        })
        .await?;

        // Wait for cancellation
        cancel.cancelled().await;
        tracing::info!("WebSocket listener stopping");
    } else {
        // Clipboard watcher
        tracing::info!("Starting clipboard watcher");

        let tx = event_tx.clone();
        tokio::select! {
            result = saya_io::clipboard::watch_clipboard(move |text| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = tx.send(AppEvent::TextInput(text)).await {
                        tracing::error!("Failed to send clipboard text to app: {}", e);
                    }
                });
            }) => {
                if let Err(e) = result {
                    tracing::error!("Clipboard watcher error: {}", e);
                }
            }
            _ = cancel.cancelled() => {
                tracing::info!("Clipboard watcher stopping");
            }
        }
    }

    Ok(())
}
