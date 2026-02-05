use std::sync::Arc;
use std::time::Duration;

use kanal::AsyncSender;
use saya_types::{AppEvent, CaptureRegion, TextSource};
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

    let (listen_to_ws, ocr_enabled, ocr_language, ocr_region, target_window, hotkey_poll_interval_ms) = {
        let config = state.config.read().await;
        (
            config.listen_to_ws,
            config.ocr.enabled,
            config.ocr.language.clone(),
            config.ocr.capture_region,
            config.ocr.target_window.clone(),
            config.hotkey_poll_interval_ms,
        )
    };

    // Spawn OCR hotkey listener if enabled
    if ocr_enabled {
        let tx = event_tx.clone();
        let cancel_clone = cancel.clone();

        let state_clone = state.clone(); // Arc<AppState> for the blocking task
        tokio::task::spawn_blocking(move || {
            tracing::debug!(">>> [OCR] Starting hotkey listener...");

            let hotkey_manager = match saya_ocr::HotkeyManager::new() {
                Ok(m) => m,
                Err(e) => {
                    tracing::debug!(">>> [OCR] Failed to create hotkey: {}", e);
                    return;
                }
            };

            tracing::debug!(">>> [OCR] F9 hotkey registered, polling...");

            loop {
                if cancel_clone.is_cancelled() {
                    break;
                }

                if hotkey_manager.poll() {
                    tracing::debug!(">>> [OCR] F9 pressed!");

                    let tx = tx.clone();
                    let target_window = target_window.clone();
                    let ocr_language = ocr_language.clone();

                    // Move Arc<AppState> into the inner blocking task
                    let state_for_task = state_clone.clone();

                    tokio::spawn(async move {
                        tracing::debug!(">>> [OCR] Starting async OCR flow...");

                        // Determine capture region
                        let region = if let Some(ref title) = target_window {
                            tracing::debug!(">>> [OCR] Target window: {}", title);
                            ocr_region.or(Some(CaptureRegion {
                                x: 100,
                                y: 100,
                                width: 600,
                                height: 400,
                            }))
                        } else if let Some(r) = ocr_region {
                            Some(r)
                        } else {
                            tracing::debug!(">>> [OCR] No capture region configured!");
                            let _ = tx
                                .send(AppEvent::OcrStatusUpdate {
                                    status: "No capture region configured".to_string(),
                                    capturing: false,
                                })
                                .await;
                            return;
                        };

                        let Some(region) = region else { return };

                        tracing::debug!(
                            ">>> [OCR] Capturing region: {}x{} at ({},{})",
                            region.width,
                            region.height,
                            region.x,
                            region.y
                        );

                        // Run OCR in spawn_blocking
                        let state_ref = state_for_task; // Arc<AppState> owns engine here
                        let result = tokio::task::spawn_blocking(move || {
                            let _com = saya_ocr::ComGuard::initialize()?;

                            let image_data = saya_ocr::capture_screen_region(region)?;
                            let text = saya_ocr::recognize_sync(
                                &state_ref.ocr_engine, // reference safe here
                                &image_data,
                                &ocr_language,
                            )?;
                            Ok::<_, anyhow::Error>((image_data.len(), text))
                        })
                        .await;

                        match result {
                            Ok(Ok((bytes, text))) => {
                                tracing::debug!(
                                    ">>> [OCR] Captured {} bytes, got text ({} chars)",
                                    bytes,
                                    text.len()
                                );
                                if !text.trim().is_empty() {
                                    let _ = tx
                                        .send(AppEvent::RawTextInput {
                                            text: text.clone(),
                                            source: TextSource::Ocr,
                                        })
                                        .await;
                                    let _ = tx.send(AppEvent::TextInput(text)).await;
                                } else {
                                    let _ = tx
                                        .send(AppEvent::OcrStatusUpdate {
                                            status: "No text found".to_string(),
                                            capturing: false,
                                        })
                                        .await;
                                }
                            }
                            Ok(Err(e)) => {
                                tracing::debug!(">>> [OCR] OCR failed: {}", e);
                            }
                            Err(e) => {
                                tracing::debug!(">>> [OCR] Task join error: {}", e);
                            }
                        }
                    });
                }

                std::thread::sleep(std::time::Duration::from_millis(hotkey_poll_interval_ms));
            }

            tracing::debug!(">>> [OCR] Hotkey listener stopped");
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
