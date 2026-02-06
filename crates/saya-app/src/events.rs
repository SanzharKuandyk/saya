use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_anki::AnkiConnectClient;
use saya_lang_japanese::{JapaneseProcessor, JapaneseTranslator};
use saya_types::AppEvent;
use trigger_auto_ocr::start_auto_ocr_loop;

use crate::ocr_context::OcrContext;
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
    processor: Arc<JapaneseProcessor>,
    translator: Arc<Option<JapaneseTranslator>>,
) -> anyhow::Result<()> {
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

    // Create OcrContext once for all OCR operations
    let ocr_ctx = OcrContext::new(
        state.clone(),
        app_to_ui_tx.clone(),
        processor.clone(),
        translator.clone(),
    );

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
            &ocr_ctx,
        )
        .await?;
    }
}

async fn handle_events(
    state: Arc<AppState>,
    event: AppEvent,
    app_to_ui_tx: &AsyncSender<AppEvent>,
    processor: &Arc<JapaneseProcessor>,
    _translator: &Arc<Option<JapaneseTranslator>>,
    anki_client: Option<&AnkiConnectClient>,
    ocr_ctx: &OcrContext,
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

            // Persist config to disk
            save_config(config.clone(), "main")?;

            // Broadcast config change to components
            app_to_ui_tx.send(AppEvent::ConfigChanged).await?;
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

            // Store current region for future use
            *state.current_capture_region.write().await = Some(region);

            handle_ocr_trigger(ocr_ctx, region, false).await?;
        }
        AppEvent::TriggerAutoOcr(region) => {
            // Store initial region for auto OCR
            *state.current_capture_region.write().await = Some(region);

            start_auto_ocr_loop(ocr_ctx, region);
        }
        AppEvent::UpdateCaptureRegion(region) => {
            // Update region while auto OCR is running (window moved/resized)
            *state.current_capture_region.write().await = Some(region);
            tracing::debug!(">>> [OCR] Region updated: {}x{} at ({}, {})",
                region.width, region.height, region.x, region.y);
        }
        AppEvent::CaptureWindow { window_id } => {
            tracing::debug!(">>> [OCR] CaptureWindow: {:?} <<<", window_id);

            handle_window_capture(ocr_ctx, window_id).await?;
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
