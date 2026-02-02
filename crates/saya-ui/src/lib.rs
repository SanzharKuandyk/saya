use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender, Receiver, Sender};
use saya_core::state::AppState;
use saya_core::types::{AppEvent, DisplayResult, TextSource, UiEvent};

slint::include_modules!();

pub async fn ui_loop(
    _state: Arc<AppState>,
    app_to_ui_rx: AsyncReceiver<AppEvent>,
    ui_to_app_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    tracing::info!("UI loop starting");

    let (sync_tx, sync_rx) = kanal::unbounded::<AppEvent>();
    let (app_sync_tx, app_sync_rx) = kanal::unbounded::<AppEvent>();

    let ui_thread = std::thread::spawn(move || run_slint_ui(sync_tx, app_sync_rx));

    let forward_to_ui = tokio::spawn({
        async move {
            tracing::info!("[UI] Starting app->ui forwarder");
            while let Ok(event) = app_to_ui_rx.recv().await {
                tracing::debug!(
                    "[UI] Forwarding app->ui: {:?}",
                    std::mem::discriminant(&event)
                );
                if app_sync_tx.send(event).is_err() {
                    break;
                }
            }
        }
    });

    tracing::info!("[UI] Forwarding events from UI to app");
    while let Ok(event) = sync_rx.as_async().recv().await {
        tracing::info!(
            "[UI] Forwarding ui->app: {:?}",
            std::mem::discriminant(&event)
        );
        if let Err(e) = ui_to_app_tx.send(event).await {
            tracing::error!("[UI] Failed to forward event: {}", e);
            break;
        }
    }

    forward_to_ui.abort();
    if let Err(e) = ui_thread.join() {
        tracing::error!("[UI] UI thread panicked: {:?}", e);
    }

    tracing::info!("UI loop exiting");
    Ok(())
}

fn run_slint_ui(
    ui_to_app_tx: Sender<AppEvent>,
    app_to_ui_rx: Receiver<AppEvent>,
) -> anyhow::Result<()> {
    tracing::info!("[SLINT] UI thread starting");

    let window = OverlayWindow::new()?;
    let window_weak = window.as_weak();

    let ocr_window = OcrWindow::new()?;
    let ocr_window_weak = ocr_window.as_weak();

    tracing::debug!("[SLINT] UI windows created");

    let window_ids = std::rc::Rc::new(std::cell::RefCell::new(Vec::<u32>::new()));

    {
        let ocr_weak = ocr_window.as_weak();
        let ids = window_ids.clone();
        ocr_window.on_refresh_windows(move || {
            if let Ok(windows) = saya_ocr::list_windows() {
                let mut stored_ids = ids.borrow_mut();
                stored_ids.clear();

                let titles: Vec<slint::SharedString> = windows
                    .iter()
                    .map(|(id, title)| {
                        stored_ids.push(*id);
                        title.chars().take(40).collect::<String>().into()
                    })
                    .collect();

                if let Some(win) = ocr_weak.upgrade() {
                    let model = std::rc::Rc::new(slint::VecModel::from(titles));
                    win.set_window_list(model.into());
                    win.set_selected_window_index(-1);
                }
            }
        });
    }

    {
        let ids = window_ids.clone();
        ocr_window.on_window_selected(move |idx| {
            let ids = ids.borrow();
            if let Some(id) = ids.get(idx as usize) {
                tracing::debug!("[SLINT] Selected window ID: {}", id);
            }
        });
    }

    {
        let tx = ui_to_app_tx.clone();
        let ocr_weak = ocr_window.as_weak();

        ocr_window.on_capture_clicked(move || {
            tracing::debug!("[SLINT] OCR capture button clicked");

            if let Some(win) = ocr_weak.upgrade() {
                win.set_is_capturing(true);
                win.set_status("Capturing...".into());

                let pos = win.window().position();
                let size = win.window().size();

                tracing::info!(
                    "[SLINT] OCR window: {}x{} at ({}, {})",
                    size.width,
                    size.height,
                    pos.x,
                    pos.y
                );

                match tx.send(AppEvent::TriggerOcr {
                    x: pos.x,
                    y: pos.y,
                    width: size.width,
                    height: size.height,
                }) {
                    Ok(_) => tracing::info!("[SLINT] TriggerOcr sent successfully"),
                    Err(e) => tracing::error!("[SLINT] Failed to send: {}", e),
                }
            }
        });
    }

    ocr_window.show()?;
    tracing::debug!("[SLINT] OCR window shown");

    let results_store = Arc::new(std::sync::Mutex::new(Vec::<DisplayResult>::new()));

    {
        let results_clone = results_store.clone();
        let tx = ui_to_app_tx.clone();
        window.on_add_to_anki(move |idx| {
            let results = results_clone.lock().unwrap();
            if let Some(result) = results.get(idx as usize) {
                let result = result.clone();
                if let Err(e) = tx.send(AppEvent::CreateCard(result)) {
                    tracing::error!("[SLINT] Failed to send CreateCard: {}", e);
                }
            }
        });
    }

    {
        let window_weak = window_weak.clone();
        let ocr_weak = ocr_window_weak.clone();
        let results_store = results_store.clone();

        std::thread::spawn(move || {
            tracing::info!("[SLINT-RX] Event receiver thread started");
            while let Ok(event) = app_to_ui_rx.recv() {
                tracing::debug!("[SLINT-RX] Received: {:?}", std::mem::discriminant(&event));

                let window_weak = window_weak.clone();
                let ocr_weak = ocr_weak.clone();
                let results_store = results_store.clone();

                let _ = slint::invoke_from_event_loop(move || match event {
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
                            tracing::debug!("[SLINT] Hooked text from {}: {} chars", source_str, text.len());
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
                            tracing::debug!(
                                "[SLINT] OCR status: {} (capturing: {})",
                                status,
                                capturing
                            );
                            w.set_status(status.into());
                            w.set_is_capturing(capturing);
                        }
                    }
                    _ => {}
                });
            }
            tracing::info!("[SLINT-RX] Event receiver thread stopped");
        });
    }

    window.show()?;
    tracing::info!("[SLINT] Running event loop");

    slint::run_event_loop()?;

    tracing::info!("[SLINT] Event loop exited");
    Ok(())
}
