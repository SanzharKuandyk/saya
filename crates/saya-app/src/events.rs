use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_core::language::LanguageProcessor;
use saya_core::types::{AppEvent, DisplayResult};
use saya_lang_japanese::{JMdict, JapaneseProcessor};

use crate::state::AppState;

/// App's main loop
pub async fn event_loop(
    state: Arc<AppState>,
    ui_to_app_rx: AsyncReceiver<AppEvent>,
    app_to_ui_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    let processor = JapaneseProcessor::new(JMdict::new());

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
        AppEvent::TextInput(text) => {
            tracing::info!("Processing text: {}", text);

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
                        });
                    }
                    break;
                }
            }

            if !display_results.is_empty() {
                app_to_ui_tx.send(AppEvent::ShowResults(display_results)).await?;
            }
        }
    }

    Ok(())
}
