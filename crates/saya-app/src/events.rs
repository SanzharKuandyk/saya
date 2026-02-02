use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_core::language::LanguageProcessor;
use saya_core::types::{AppEvent, DisplayResult};
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

    loop {
        match ui_to_app_rx.recv().await {
            Ok(event) => {
                println!(">>> EVENT RECEIVED: {:?} <<<", std::mem::discriminant(&event));
                handle_events(
                    state.clone(),
                    &processor,
                    anki_client.as_ref(),
                    &app_to_ui_tx,
                    event,
                )
                .await?;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

async fn handle_events(
    state: Arc<AppState>,
    processor: &JapaneseProcessor,
    anki_client: Option<&saya_anki::AnkiConnectClient>,
    app_to_ui_tx: &AsyncSender<AppEvent>,
    event: AppEvent,
) -> anyhow::Result<()> {
    println!(">>> HANDLING EVENT <<<");
    match event {
        AppEvent::ConfigChanged => {}
        AppEvent::UiEvent(_event) => {}
        AppEvent::ApiRequest(_event) => {}
        AppEvent::ShowResults(_) => {}
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
        AppEvent::TriggerOcr { x, y, width, height } => {
            println!(">>> [OCR] TriggerOcr: region=({},{}) {}x{} <<<", x, y, width, height);

            let state = state.clone();
            let app_to_ui_tx = app_to_ui_tx.clone();

            tokio::spawn(async move {
                // Get OCR language from config
                let ocr_language = {
                    let config = state.config.read().await;
                    config.ocr.language.clone()
                };

                let region = saya_ocr::CaptureRegion { x, y, width, height };
                println!(">>> [OCR] Capturing region: {}x{} at ({},{})", width, height, x, y);

                // Run capture and OCR in blocking context
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
                        println!(">>> [OCR] Captured {} bytes, got text ({} chars)", bytes, text.len());

                        if !text.trim().is_empty() {
                            match app_to_ui_tx.send(AppEvent::TextInput(text)).await {
                                Ok(_) => println!(">>> [OCR] TextInput sent!"),
                                Err(e) => println!(">>> [OCR] Send failed: {}", e),
                            }
                        } else {
                            println!(">>> [OCR] Empty text after OCR");
                            let _ = app_to_ui_tx.send(AppEvent::OcrStatusUpdate {
                                status: "No text found".to_string(),
                                capturing: false,
                            }).await;
                        }
                    }
                    Ok(Err(e)) => {
                        println!(">>> [OCR] OCR failed: {}", e);
                        let _ = app_to_ui_tx.send(AppEvent::OcrStatusUpdate {
                            status: format!("OCR failed: {}", e),
                            capturing: false,
                        }).await;
                    }
                    Err(e) => {
                        println!(">>> [OCR] Task join error: {}", e);
                    }
                }
            });
        }
        AppEvent::CaptureWindow { window_id } => {
            println!(">>> [OCR] CaptureWindow: {:?} <<<", window_id);

            let state = state.clone();
            let app_to_ui_tx = app_to_ui_tx.clone();

            tokio::spawn(async move {
                // Get OCR language from config
                let ocr_language = {
                    let config = state.config.read().await;
                    config.ocr.language.clone()
                };

                // Get region from config or use default
                let region = {
                    let config = state.config.read().await;
                    config.ocr.capture_region.map(|r| saya_ocr::CaptureRegion {
                        x: r.x, y: r.y, width: r.width, height: r.height
                    }).or(Some(saya_ocr::CaptureRegion {
                        x: 100, y: 100, width: 600, height: 400
                    }))
                };

                let Some(region) = region else {
                    let _ = app_to_ui_tx.send(AppEvent::OcrStatusUpdate {
                        status: "No capture region configured".to_string(),
                        capturing: false,
                    }).await;
                    return;
                };

                println!(">>> [OCR] Capturing region: {}x{} at ({},{})", region.width, region.height, region.x, region.y);

                // Run capture and OCR in blocking context
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
                        println!(">>> [OCR] Captured {} bytes, got text ({} chars)", bytes, text.len());

                        if !text.trim().is_empty() {
                            match app_to_ui_tx.send(AppEvent::TextInput(text)).await {
                                Ok(_) => println!(">>> [OCR] TextInput sent!"),
                                Err(e) => println!(">>> [OCR] Send failed: {}", e),
                            }
                        } else {
                            println!(">>> [OCR] Empty text after OCR");
                            let _ = app_to_ui_tx.send(AppEvent::OcrStatusUpdate {
                                status: "No text found".to_string(),
                                capturing: false,
                            }).await;
                        }
                    }
                    Ok(Err(e)) => {
                        println!(">>> [OCR] OCR failed: {}", e);
                        let _ = app_to_ui_tx.send(AppEvent::OcrStatusUpdate {
                            status: format!("OCR failed: {}", e),
                            capturing: false,
                        }).await;
                    }
                    Err(e) => {
                        println!(">>> [OCR] Task join error: {}", e);
                    }
                }
            });
        }
        AppEvent::OcrStatusUpdate { status, capturing } => {
            tracing::info!("OCR status: {} (capturing: {})", status, capturing);
        }
        AppEvent::TextInput(text) => {
            eprintln!(">>> [EVENT] TextInput received: '{}' <<<", text);
            tracing::info!("Processing text: {}", text);

            let normalized = processor.normalize(&text);
            eprintln!(">>> [EVENT] Normalized: '{}' <<<", normalized);

            let tokens = processor.tokenize(&normalized);
            eprintln!(">>> [EVENT] Tokens: {:?} <<<", tokens);

            let mut display_results = Vec::new();

            for token in tokens.iter().take(10) {
                let results = processor.lookup(token);
                eprintln!(">>> [EVENT] Lookup token '{:?}': {} results <<<", token, results.len());
                if !results.is_empty() {
                    eprintln!(">>> [EVENT] Found {} results for token '{:?}' <<<", results.len(), token);
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
                    // Continue loop to find results for all matching tokens
                }
            }

            eprintln!(">>> [EVENT] Display results count: {} <<<", display_results.len());

            if !display_results.is_empty() {
                eprintln!(">>> [EVENT] Sending ShowResults <<<");
                app_to_ui_tx.send(AppEvent::ShowResults(display_results)).await?;
            } else {
                eprintln!(">>> [EVENT] No results found for input text");
                eprintln!(">>> [EVENT] Hint: This is a Japanese dictionary (JMdict)");
                eprintln!(">>> [EVENT] OCR captured: '{}'", text);
                let _ = app_to_ui_tx.send(AppEvent::OcrStatusUpdate {
                    status: "Japanese text only".to_string(),
                    capturing: false,
                }).await;
            }
        }
    }

    Ok(())
}
