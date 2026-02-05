use std::sync::Arc;
use std::sync::atomic::Ordering;

use kanal::AsyncSender;
use saya_lang_japanese::{JapaneseProcessor, JapaneseTranslator};
use saya_types::{AppEvent, CaptureRegion};

use crate::AppState;

use super::trigger_ocr::handle_ocr_trigger;

pub fn start_auto_ocr_loop(
    state: Arc<AppState>,
    region: CaptureRegion,
    app_to_ui_tx: AsyncSender<AppEvent>,
    processor: Arc<JapaneseProcessor>,
    translator: Arc<Option<JapaneseTranslator>>,
) {
    // Don't start again if already running
    if state.auto_ocr_running.swap(true, Ordering::SeqCst) {
        return;
    }

    let state_clone = state.clone();
    let processor_clone = processor.clone();
    let translator_clone = translator.clone();
    let tx_clone = app_to_ui_tx.clone();

    tokio::spawn(async move {
        loop {
            let auto_enabled = {
                let config = state_clone.config.read().await;
                config.ocr.auto
            };

            if !auto_enabled {
                state_clone.auto_ocr_running.store(false, Ordering::SeqCst);
                break;
            }

            // Run one OCR cycle
            let _ = handle_ocr_trigger(
                state_clone.clone(),
                region,
                &tx_clone,
                &processor_clone,
                &translator_clone,
                true,
            )
            .await;

            tokio::task::yield_now().await;
            // Prevent CPU burn
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });
}
