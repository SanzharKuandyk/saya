use saya_core::language::LanguageProcessor;
use saya_types::{AppEvent, DisplayResult, TextSource};
use saya_translator::Translator;

use crate::ocr_context::OcrContext;

pub async fn handle_window_capture(
    ctx: &OcrContext,
    window_id: Option<u32>,
) -> anyhow::Result<()> {
    let state = &ctx.state;
    let app_to_ui_tx = &ctx.event_tx;
    let processor = &ctx.processor;
    let translator = &ctx.translator;
    let ocr_language = {
        let config = state.config.read().await;
        config.ocr.language.clone()
    };

    let state_clone = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        let _com = saya_ocr::ComGuard::initialize()?;

        let image_data = if let Some(id) = window_id {
            tracing::debug!(">>> [OCR] Capturing window {}", id);
            saya_ocr::capture_window(id)?
        } else {
            tracing::debug!(">>> [OCR] Capturing primary screen");
            saya_ocr::capture_primary_screen()?
        };

        tracing::debug!(">>> [OCR] Captured {} bytes", image_data.len());
        let text = saya_ocr::recognize_sync(&state_clone.ocr_engine, &image_data, &ocr_language)?;
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
                    app_to_ui_tx
                        .send(AppEvent::ShowResults(display_results))
                        .await?;
                }

                // Translation
                if let Some(t) = (**translator).as_ref() {
                    let config = state.config.read().await;
                    let from = config.translator.from_lang.clone();
                    let to = config.translator.to_lang.clone();
                    drop(config);

                    match t.translate(&text, from.clone(), to.clone()).await {
                        Ok(translation) => {
                            let _ = app_to_ui_tx
                                .send(AppEvent::ShowTranslation {
                                    text: translation.text,
                                    from_lang: from,
                                    to_lang: to,
                                })
                                .await;
                        }
                        Err(e) => {
                            tracing::warn!("Translation failed: {}", e);
                        }
                    }
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

    Ok(())
}
