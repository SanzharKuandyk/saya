use std::sync::Arc;
use std::time::Duration;

use kanal::AsyncSender;
use saya_core::types::{AppEvent, TextSource};
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

pub async fn watcher_io(
    state: Arc<AppState>,
    _delta_time: Duration,
    cancel: CancellationToken,
    event_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    tracing::info!("watcher_io started");

    let (listen_to_ws, ocr_enabled, ocr_language, ocr_region, target_window) = {
        let config = state.config.read().await;
        (
            config.listen_to_ws,
            config.ocr.enabled,
            config.ocr.language.clone(),
            config.ocr.capture_region,
            config.ocr.target_window.clone(),
        )
    };

    // Spawn OCR hotkey listener if enabled
    if ocr_enabled {
        let tx = event_tx.clone();
        let cancel_clone = cancel.clone();

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
                    let ocr_region = ocr_region.clone();
                    let target_window = target_window.clone();
                    let ocr_language = ocr_language.clone();

                    tokio::spawn(async move {
                        tracing::debug!(">>> [OCR] Starting async OCR flow...");

                        // Get capture region from config
                        let region = if let Some(ref title) = target_window {
                            // TODO: Capture specific window - for now use region
                            tracing::debug!(">>> [OCR] Target window: {}", title);
                            ocr_region.map(|r| saya_ocr::CaptureRegion {
                                x: r.x, y: r.y, width: r.width, height: r.height
                            }).or(Some(saya_ocr::CaptureRegion {
                                x: 100, y: 100, width: 600, height: 400
                            }))
                        } else if let Some(r) = ocr_region {
                            Some(saya_ocr::CaptureRegion {
                                x: r.x, y: r.y, width: r.width, height: r.height
                            })
                        } else {
                            tracing::debug!(">>> [OCR] No capture region configured!");
                            let _ = tx.send(AppEvent::OcrStatusUpdate {
                                status: "No capture region configured".to_string(),
                                capturing: false,
                            }).await;
                            return;
                        };

                        let Some(region) = region else { return };

                        tracing::debug!(">>> [OCR] Capturing region: {}x{} at ({},{})",
                            region.width, region.height, region.x, region.y);

                        // Run capture and OCR in blocking context (Windows COM)
                        let result = tokio::task::spawn_blocking(move || {
                            // Initialize COM for this thread (required for OCR)
                            unsafe { windows::Win32::System::Com::CoInitializeEx(
                                Some(std::ptr::null()),
                                windows::Win32::System::Com::COINIT_MULTITHREADED,
                            ) }.ok()?;

                            let image_data = saya_ocr::capture_screen_region(region)?;
                            let text = saya_ocr::recognize_sync(&image_data, &ocr_language)?;
                            Ok::<_, anyhow::Error>((image_data.len(), text))
                        }).await;

                        match result {
                            Ok(Ok((bytes, text))) => {
                                tracing::debug!(">>> [OCR] Captured {} bytes, got text ({} chars)", bytes, text.len());

                                if !text.trim().is_empty() {
                                    let _ = tx.send(AppEvent::RawTextInput {
                                        text: text.clone(),
                                        source: TextSource::Ocr,
                                    }).await;
                                    tracing::debug!(">>> [OCR] Sending TextInput event...");
                                    match tx.send(AppEvent::TextInput(text)).await {
                                        Ok(_) => tracing::debug!(">>> [OCR] TextInput sent!"),
                                        Err(e) => tracing::debug!(">>> [OCR] Send failed: {}", e),
                                    }
                                } else {
                                    tracing::debug!(">>> [OCR] Empty text after OCR");
                                    let _ = tx.send(AppEvent::OcrStatusUpdate {
                                        status: "No text found".to_string(),
                                        capturing: false,
                                    }).await;
                                }
                            }
                            Ok(Err(e)) => {
                                tracing::debug!(">>> [OCR] OCR failed: {}", e);
                                let _ = tx.send(AppEvent::OcrStatusUpdate {
                                    status: format!("OCR failed: {}", e),
                                    capturing: false,
                                }).await;
                            }
                            Err(e) => {
                                tracing::debug!(">>> [OCR] Task join error: {}", e);
                            }
                        }
                    });
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
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
                let _ = tx.send(AppEvent::RawTextInput {
                    text: text.clone(),
                    source: TextSource::Websocket,
                }).await;
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
