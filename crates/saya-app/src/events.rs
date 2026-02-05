use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_anki::AnkiConnectClient;
use saya_lang_japanese::{JapaneseProcessor, JapaneseTranslator};
use saya_types::AppEvent;
use trigger_auto_ocr::start_auto_ocr_loop;

use crate::profile::{save_config, update_config_field};
use crate::state::AppState;

pub mod capture_window;
pub mod create_card;
pub mod text_input;
pub mod trigger_auto_ocr;
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
    let processor = Arc::new(processor);
    let translator = Arc::new(translator);
    loop {
        tracing::info!("[EVENT_LOOP] Calling recv().await...");
        let event = ui_to_app_rx.recv().await?;

        tracing::info!(
            "[EVENT_LOOP] EVENT RECEIVED: {:?}",
            std::mem::discriminant(&event)
        );

        handle_events(
            state.clone(),
            event,
            &app_to_ui_tx,
            &processor,
            &translator,
            anki_client.as_ref(),
        )
        .await?;
    }
}

async fn handle_events(
    state: Arc<AppState>,
    event: AppEvent,
    app_to_ui_tx: &AsyncSender<AppEvent>,
    processor: &Arc<JapaneseProcessor>,
    translator: &Arc<Option<JapaneseTranslator>>,
    anki_client: Option<&AnkiConnectClient>,
) -> anyhow::Result<()> {
    tracing::debug!(">>> HANDLING EVENT <<<");
    match event {
        // DO REALLY NEED CONFIG CHANGED?
        AppEvent::ConfigChanged => {
            // ConfigChanged is broadcast to components after updates are processed
            // Components will handle their specific config changes in their own logic
        }
        AppEvent::ConfigUpdate { field, value } => {
            tracing::info!("Config update: {} = {}", field, value);

            let mut config = state.config.write().await;

            update_config_field(&mut config, &field, &value)?;

            // NEED TO SAVE CONFIG HERE, NEED TO PROPERLY think about this
        }
        AppEvent::UiEvent(_event) => {}
        AppEvent::ApiRequest(_event) => {}
        AppEvent::ShowResults(_) => {}
        AppEvent::RawTextInput { text: _, source: _ } => {
            // RawTextInput events are handled by UI layer, no processing needed here
        }
        AppEvent::CreateCard(result) => {
            // Anki Card Creation
            handle_card_creation(state, result, anki_client).await?;
        }
        AppEvent::TriggerOcr(region) => {
            tracing::debug!(">>> [OCR] Triggered");

            handle_ocr_trigger(state, region, app_to_ui_tx, processor, translator, false).await?;
        }
        AppEvent::TriggerAutoOcr(region) => {
            start_auto_ocr_loop(
                state,
                region,
                app_to_ui_tx.clone(),
                processor.clone(),
                translator.clone(),
            );
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
