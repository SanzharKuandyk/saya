use std::sync::atomic::Ordering;

use saya_types::CaptureRegion;

use crate::ocr_context::OcrContext;

use super::trigger_ocr::handle_ocr_trigger;

pub fn start_auto_ocr_loop(ctx: &OcrContext, region: CaptureRegion) {
    let state = &ctx.state;

    // Don't start again if already running
    if state.auto_ocr_running.swap(true, Ordering::SeqCst) {
        return;
    }

    let ctx_clone = ctx.clone();

    tokio::spawn(async move {
        loop {
            let (auto_enabled, interval_ms) = {
                let config = ctx_clone.state.config.read().await;
                (config.ocr.auto, config.auto_ocr_interval_ms)
            };

            if !auto_enabled {
                ctx_clone.state.auto_ocr_running.store(false, Ordering::SeqCst);
                break;
            }

            // Run one OCR cycle
            let _ = handle_ocr_trigger(&ctx_clone, region, true).await;

            tokio::task::yield_now().await;
            tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
        }
    });
}
