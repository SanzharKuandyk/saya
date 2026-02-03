use kanal::AsyncSender;
use saya_core::language::LanguageProcessor;
use saya_core::types::{AppEvent, DisplayResult};
use saya_lang_japanese::JapaneseProcessor;

pub async fn handle_text_input(
    text: String,
    processor: &JapaneseProcessor,
    app_to_ui_tx: &AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
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

    Ok(())
}
