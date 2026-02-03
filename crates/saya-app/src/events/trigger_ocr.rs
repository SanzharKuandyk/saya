use std::sync::Arc;

use kanal::AsyncSender;
use saya_core::language::LanguageProcessor;
use saya_core::types::{AppEvent, CaptureRegion, DisplayResult, TextSource};
use saya_lang_japanese::{JapaneseProcessor, JapaneseTranslator};
use saya_translator::Translator;

use crate::AppState;

pub async fn handle_ocr_trigger(
    state: Arc<AppState>,
    region: CaptureRegion,
    app_to_ui_tx: &AsyncSender<AppEvent>,
    processor: &JapaneseProcessor,
    translator: Option<&JapaneseTranslator>,
) -> anyhow::Result<()> {
    let ocr_language = {
        let config = state.config.read().await;
        config.ocr.language.clone()
    };

    let region = CaptureRegion {
        x: region.x,
        y: region.y,
        width: region.width,
        height: region.height,
    };

    let state_clone = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                Some(std::ptr::null()),
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            )
        }
        .ok()?;

        let image_data = saya_ocr::capture_screen_region(region)?;
        let text = saya_ocr::recognize_sync(&state_clone.ocr_engine, &image_data, &ocr_language)?;
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
                    let _ = app_to_ui_tx
                        .send(AppEvent::ShowResults(display_results))
                        .await;
                }

                // Translation
                if let Some(t) = translator {
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
