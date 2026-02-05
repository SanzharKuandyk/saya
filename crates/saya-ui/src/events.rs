use std::sync::Mutex;

use saya_types::{AppEvent, DisplayResult, TextSource, UiEvent};
use slint::{ComponentHandle, Weak};

use crate::{DictResult, OcrWindow, OverlayWindow};

pub fn handle_events(
    event: AppEvent,
    window_weak: Weak<OverlayWindow>,
    ocr_weak: Weak<OcrWindow>,
    results_store: &Mutex<Vec<DisplayResult>>,
) {
    match event {
        AppEvent::UiEvent(UiEvent::Show) => {
            if let Some(w) = window_weak.upgrade() {
                let _ = w.show();
                tracing::debug!("[SLINT] Overlay shown");
            }
        }
        AppEvent::UiEvent(UiEvent::Hide) => {
            if let Some(w) = window_weak.upgrade() {
                let _ = w.hide();
                tracing::debug!("[SLINT] Overlay hidden");
            }
        }
        AppEvent::UiEvent(UiEvent::Close) => {
            if let Some(w) = window_weak.upgrade() {
                let _ = w.hide();
            }
            slint::quit_event_loop().ok();
        }
        AppEvent::RawTextInput { text, source } => {
            if let Some(w) = window_weak.upgrade() {
                let source_str = match source {
                    TextSource::Ocr => "OCR",
                    TextSource::Clipboard => "Clipboard",
                    TextSource::Websocket => "WebSocket",
                    TextSource::Manual => "Manual",
                };
                tracing::debug!(
                    "[SLINT] Hooked text from {}: {} chars",
                    source_str,
                    text.len()
                );
                w.set_hooked_text(text.into());
                w.set_text_source(source_str.into());
                w.show().ok();
            }
        }
        AppEvent::ShowResults(results) => {
            if let Some(w) = window_weak.upgrade() {
                tracing::debug!("[SLINT] Showing {} results", results.len());
                *results_store.lock().unwrap() = results.clone();

                let slint_results: Vec<DictResult> = results
                    .into_iter()
                    .map(|r| DictResult {
                        term: r.term.into(),
                        reading: r.reading.into(),
                        definition: r.definition.into(),
                        frequency: r.frequency.unwrap_or_default().into(),
                        pitch_accent: r.pitch_accent.unwrap_or_default().into(),
                        jlpt_level: r.jlpt_level.unwrap_or_default().into(),
                        conjugation: r.conjugation.unwrap_or_default().into(),
                    })
                    .collect();

                let model = std::rc::Rc::new(slint::VecModel::from(slint_results));
                w.set_results(model.into());
                w.show().ok();
            }
        }
        AppEvent::OcrStatusUpdate { status, capturing } => {
            if let Some(w) = ocr_weak.upgrade() {
                tracing::debug!("[SLINT] OCR status: {} (capturing: {})", status, capturing);
                w.set_status(status.into());
                w.set_is_capturing(capturing);
            }
        }
        AppEvent::BackendReady => {
            if let Some(w) = ocr_weak.upgrade() {
                tracing::debug!("[SLINT] Backend ready");
                w.set_is_ready(true);
                w.set_status("Ready".into());
            }
        }
        AppEvent::ShowTranslation {
            text,
            from_lang,
            to_lang,
        } => {
            if let Some(w) = window_weak.upgrade() {
                tracing::debug!("[SLINT] Translation: {} -> {}", from_lang, to_lang);
                w.set_translation(text.into());
            }
        }
        _ => {}
    };
}
