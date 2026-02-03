use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_core::types::AppEvent;
use saya_lang_japanese::{JapaneseProcessor, JapaneseTranslator};

use crate::state::AppState;

pub mod capture_window;
pub mod create_card;
pub mod text_input;
pub mod trigger_ocr;

use capture_window::handle_window_capture;
use create_card::handle_card_creation;
use text_input::handle_text_input;
use trigger_ocr::handle_ocr_trigger;

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

    // Initialize translator
    let translator = {
        let config = state.config.read().await;
        if config.translator.enabled && !config.translator.api_key.is_empty() {
            Some(JapaneseTranslator::new(
                config.translator.api_key.clone(),
                config.translator.api_url.clone(),
            ))
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
            translator.as_ref(),
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
    translator: Option<&JapaneseTranslator>,
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
            // Anki Card Creation
            handle_card_creation(state, result, anki_client).await?;
        }
        AppEvent::TriggerOcr(region) => {
            tracing::debug!(">>> [OCR] Triggered");

            handle_ocr_trigger(state, region, app_to_ui_tx, processor, translator).await?;
        }
        AppEvent::CaptureWindow { window_id } => {
            tracing::debug!(">>> [OCR] CaptureWindow: {:?} <<<", window_id);

            handle_window_capture(state, window_id, app_to_ui_tx, processor, translator).await?;
        }
        AppEvent::OcrStatusUpdate { status, capturing } => {
            tracing::info!("OCR status: {} (capturing: {})", status, capturing);
        }
        AppEvent::TextInput(text) => {
            tracing::debug!("TextInput received: '{}' chars", text.len());
            tracing::info!("Processing text: {}", text);

            handle_text_input(text, processor, app_to_ui_tx).await?;
        }
        AppEvent::BackendReady => {
            // UI-only event, ignore in backend
        }
        AppEvent::ShowTranslation { .. } => {
            // UI-only event, ignore in backend
        }
    }

    Ok(())
}
