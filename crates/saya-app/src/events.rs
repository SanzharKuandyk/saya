use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_core::language::LanguageProcessor;
use saya_core::types::{AppEvent, DisplayResult, TextSource};
use saya_lang_japanese::JapaneseProcessor;

use crate::state::AppState;

/// App's main loop
pub async fn event_loop(
    state: Arc<AppState>,
    ui_to_app_rx: AsyncReceiver<AppEvent>,
    app_to_ui_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    // Initialize processor with dictionary config
    let processor = {
        let config = state.config.read().await;
        if config.dictionary.enabled {
            JapaneseProcessor::with_additional_dicts(&config.dictionary.additional_paths)
        } else {
            tracing::warn!("Dictionary disabled, using empty processor");
            JapaneseProcessor::with_additional_dicts(&[])
        }
    };

    // Initialize Anki client
    let anki_client = {
        let config = state.config.read().await;
        if config.anki.enabled {
            Some(saya_anki::AnkiConnectClient::new(config.anki.url.clone()))
        } else {
            None
        }
    };

    tracing::info!("[EVENT_LOOP] Starting main loop, waiting for events");
    loop {
        tracing::info!("[EVENT_LOOP] Calling recv().await...");
        let event = ui_to_app_rx.recv().await?;

        tracing::info!(
            "[EVENT_LOOP] EVENT RECEIVED: {:?}",
            std::mem::discriminant(&event)
        );
        handle_events(
            state.clone(),
            &processor,
            anki_client.as_ref(),
            &app_to_ui_tx,
            event,
        )
        .await?;
    }
}

async fn handle_events(
    state: Arc<AppState>,
    processor: &JapaneseProcessor,
    anki_client: Option<&saya_anki::AnkiConnectClient>,
    app_to_ui_tx: &AsyncSender<AppEvent>,
    event: AppEvent,
) -> anyhow::Result<()> {
    tracing::debug!(">>> HANDLING EVENT <<<");
    match event {
        AppEvent::ConfigChanged => {}
        AppEvent::UiEvent(_event) => {}
        AppEvent::ApiRequest(_event) => {}
        AppEvent::ShowResults(_) => {}
        AppEvent::RawTextInput { text: _, source: _ } => {
            // RawTextInput events are handled by the UI layer, no processing needed here
        }
        AppEvent::CreateCard(result) => {
            if let Some(client) = anki_client {
                let config = state.config.read().await;
                let template = saya_anki::CardTemplate::new(
                    config.anki.deck.clone(),
                    config.anki.model.clone(),
                    "{term}\n{reading}".to_string(),
                    "{definition}".to_string(),
                );

                match saya_anki::add_card(
                    client,
                    &template,
                    &result.term,
                    &result.reading,
                    &result.definition,
                )
                .await
                {
                    Ok(note_id) => {
                        tracing::info!("Added card to Anki: note_id={}", note_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to add card to Anki: {}", e);
                    }
                }
            } else {
                tracing::warn!("Anki integration disabled");
            }
        }
        AppEvent::TriggerOcr {
            x,
            y,
            width,
            height,
        } => {
            tracing::debug!(
                ">>> [OCR] TriggerOcr: region=({},{}) {}x{} <<<",
                x,
                y,
                width,
                height
            );

            let ocr_language = {
                let config = state.config.read().await;
                config.ocr.language.clone()
            };

            let region = saya_ocr::CaptureRegion {
                x,
                y,
                width,
                height,
            };

            let result = tokio::task::spawn_blocking(move || {
                unsafe {
                    windows::Win32::System::Com::CoInitializeEx(
                        Some(std::ptr::null()),
                        windows::Win32::System::Com::COINIT_MULTITHREADED,
                    )
                }
                .ok()?;

                let image_data = saya_ocr::capture_screen_region(region)?;
                let text = saya_ocr::recognize_sync(&image_data, &ocr_language)?;
                Ok::<_, anyhow::Error>(text)
            })
            .await;

            match result {
                Ok(Ok(text)) => {
                    tracing::debug!(">>> [OCR] Got text: {} chars", text.len());

                    if !text.trim().is_empty() {
                        // Show raw text
                        let _ = app_to_ui_tx
                            .send(AppEvent::RawTextInput {
                                text: text.clone(),
                                source: TextSource::Ocr,
                            })
                            .await;

                        // Dictionary processing
                        let normalized = processor.normalize(&text);
                        let tokens = processor.tokenize(&normalized);
                        let mut display_results = Vec::new();

                        for token in tokens.iter().take(10) {
                            let results = processor.lookup(token);
                            if !results.is_empty() {
                                for result in results.iter().take(5) {
                                    display_results.push(DisplayResult {
                                        term: result.term.clone(),
                                        reading: result.readings.join(", "),
                                        definition: result.definitions.join("; "),
                                        frequency: result.metadata.get("frequency_stars").cloned(),
                                        pitch_accent: result.metadata.get("pitch_accent").cloned(),
                                        jlpt_level: result.metadata.get("jlpt_level").cloned(),
                                        conjugation: result.metadata.get("conjugation").cloned(),
                                    });
                                }
                            }
                        }

                        if !display_results.is_empty() {
                            let _ = app_to_ui_tx.send(AppEvent::ShowResults(display_results)).await;
                        }

                        let _ = app_to_ui_tx
                            .send(AppEvent::OcrStatusUpdate {
                                status: "Ready".to_string(),
                                capturing: false,
                            })
                            .await;
                    } else {
                        let _ = app_to_ui_tx
                            .send(AppEvent::OcrStatusUpdate {
                                status: "No text found".to_string(),
                                capturing: false,
                            })
                            .await;
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!(">>> [OCR] Failed: {}", e);
                    let _ = app_to_ui_tx
                        .send(AppEvent::OcrStatusUpdate {
                            status: format!("Failed: {}", e),
                            capturing: false,
                        })
                        .await;
                }
                Err(e) => {
                    tracing::error!(">>> [OCR] Task error: {}", e);
                    let _ = app_to_ui_tx
                        .send(AppEvent::OcrStatusUpdate {
                            status: "Error".to_string(),
                            capturing: false,
                        })
                        .await;
                }
            }
        }
        AppEvent::CaptureWindow { window_id } => {
            tracing::debug!(">>> [OCR] CaptureWindow: {:?} <<<", window_id);

            let ocr_language = {
                let config = state.config.read().await;
                config.ocr.language.clone()
            };

            let result = tokio::task::spawn_blocking(move || {
                unsafe {
                    windows::Win32::System::Com::CoInitializeEx(
                        Some(std::ptr::null()),
                        windows::Win32::System::Com::COINIT_MULTITHREADED,
                    )
                }
                .ok()?;

                let image_data = if let Some(id) = window_id {
                    tracing::debug!(">>> [OCR] Capturing window {}", id);
                    saya_ocr::capture_window(id)?
                } else {
                    tracing::debug!(">>> [OCR] Capturing primary screen");
                    saya_ocr::capture_primary_screen()?
                };

                tracing::debug!(">>> [OCR] Captured {} bytes", image_data.len());
                let text = saya_ocr::recognize_sync(&image_data, &ocr_language)?;
                Ok::<_, anyhow::Error>(text)
            })
            .await;

            match result {
                Ok(Ok(text)) => {
                    tracing::debug!(">>> [OCR] Got text: {} chars", text.len());

                    if !text.trim().is_empty() {
                        // Show raw text in UI
                        let _ = app_to_ui_tx
                            .send(AppEvent::RawTextInput {
                                text: text.clone(),
                                source: TextSource::Ocr,
                            })
                            .await;

                        // Process dictionary
                        let normalized = processor.normalize(&text);
                        let tokens = processor.tokenize(&normalized);
                        let mut display_results = Vec::new();

                        for token in tokens.iter().take(10) {
                            let results = processor.lookup(token);
                            if !results.is_empty() {
                                for result in results.iter().take(5) {
                                    display_results.push(DisplayResult {
                                        term: result.term.clone(),
                                        reading: result.readings.join(", "),
                                        definition: result.definitions.join("; "),
                                        frequency: result.metadata.get("frequency_stars").cloned(),
                                        pitch_accent: result.metadata.get("pitch_accent").cloned(),
                                        jlpt_level: result.metadata.get("jlpt_level").cloned(),
                                        conjugation: result.metadata.get("conjugation").cloned(),
                                    });
                                }
                            }
                        }

                        if !display_results.is_empty() {
                            app_to_ui_tx.send(AppEvent::ShowResults(display_results)).await?;
                        }

                        let _ = app_to_ui_tx
                            .send(AppEvent::OcrStatusUpdate {
                                status: "Ready".to_string(),
                                capturing: false,
                            })
                            .await;
                    } else {
                        let _ = app_to_ui_tx
                            .send(AppEvent::OcrStatusUpdate {
                                status: "No text found".to_string(),
                                capturing: false,
                            })
                            .await;
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!(">>> [OCR] Failed: {}", e);
                    let _ = app_to_ui_tx
                        .send(AppEvent::OcrStatusUpdate {
                            status: format!("Failed: {}", e),
                            capturing: false,
                        })
                        .await;
                }
                Err(e) => {
                    tracing::error!(">>> [OCR] Task error: {}", e);
                    let _ = app_to_ui_tx
                        .send(AppEvent::OcrStatusUpdate {
                            status: "Error".to_string(),
                            capturing: false,
                        })
                        .await;
                }
            }
        }
        AppEvent::OcrStatusUpdate { status, capturing } => {
            tracing::info!("OCR status: {} (capturing: {})", status, capturing);
        }
        AppEvent::TextInput(text) => {
            tracing::debug!("TextInput received: '{}' chars", text.len());
            tracing::info!("Processing text: {}", text);

            let normalized = processor.normalize(&text);
            tracing::debug!("Normalized: '{}'", normalized);

            let tokens = processor.tokenize(&normalized);
            tracing::debug!("Tokenized into {} tokens", tokens.len());

            let mut display_results = Vec::new();

            for token in tokens.iter().take(10) {
                let results = processor.lookup(token);
                tracing::debug!("Token '{:?}': {} results", token, results.len());
                if !results.is_empty() {
                    for result in results.iter().take(5) {
                        display_results.push(DisplayResult {
                            term: result.term.clone(),
                            reading: result.readings.join(", "),
                            definition: result.definitions.join("; "),
                            frequency: result.metadata.get("frequency_stars").cloned(),
                            pitch_accent: result.metadata.get("pitch_accent").cloned(),
                            jlpt_level: result.metadata.get("jlpt_level").cloned(),
                            conjugation: result.metadata.get("conjugation").cloned(),
                        });
                    }
                }
            }

            tracing::debug!("Total display results: {}", display_results.len());

            if !display_results.is_empty() {
                tracing::debug!("Sending ShowResults event");
                app_to_ui_tx
                    .send(AppEvent::ShowResults(display_results))
                    .await?;
            } else {
                tracing::debug!("No results found for input text");
                let _ = app_to_ui_tx
                    .send(AppEvent::OcrStatusUpdate {
                        status: "Japanese text only".to_string(),
                        capturing: false,
                    })
                    .await;
            }
        }
        AppEvent::BackendReady => {
            // UI-only event, ignore in backend
        }
    }

    Ok(())
}
