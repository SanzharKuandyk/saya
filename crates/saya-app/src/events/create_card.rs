use std::sync::Arc;

use saya_core::types::DisplayResult;

use crate::AppState;

pub async fn handle_card_creation(
    state: Arc<AppState>,
    result: DisplayResult,
    anki_client: Option<&saya_anki::AnkiConnectClient>,
) -> anyhow::Result<()> {
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

    Ok(())
}
