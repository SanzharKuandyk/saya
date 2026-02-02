use std::sync::Arc;

use kanal::{AsyncReceiver, AsyncSender};
use saya_core::state::AppState;
use saya_core::types::{AppEvent, DisplayResult, UiEvent};

slint::include_modules!();

pub async fn ui_loop(
    _state: Arc<AppState>,
    app_to_ui_rx: AsyncReceiver<AppEvent>,
    ui_to_app_tx: AsyncSender<AppEvent>,
) -> anyhow::Result<()> {
    // Create the overlay window
    let window = OverlayWindow::new()?;
    let window_weak = window.as_weak();

    // Create OCR window
    let ocr_window = OcrWindow::new()?;
    let ocr_window_weak = ocr_window.as_weak();

    // Store window IDs for selection
    let window_ids = std::rc::Rc::new(std::cell::RefCell::new(Vec::<u32>::new()));

    // Set up refresh windows callback
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

    // Set up window selection callback
    {
        let ids = window_ids.clone();
        ocr_window.on_window_selected(move |idx| {
            let ids = ids.borrow();
            if let Some(id) = ids.get(idx as usize) {
                println!(">>> Selected window ID: {} <<<", id);
            }
        });
    }

    // Set up OCR capture callback
    {
        let tx = ui_to_app_tx.clone();
        let ids = window_ids.clone();
        ocr_window.on_capture_clicked(move || {
            println!(">>> OCR BUTTON CLICKED <<<");
            let tx = tx.clone();
            let ids = ids.clone();

            if let Some(win) = ocr_window_weak.upgrade() {
                win.set_is_capturing(true);
                win.set_status("Capturing...".into());

                let selected_idx = win.get_selected_window_index();
                let window_id = if selected_idx >= 0 {
                    ids.borrow().get(selected_idx as usize).copied()
                } else {
                    None
                };

                slint::spawn_local(async move {
                    println!(">>> Sending capture event, window_id: {:?} <<<", window_id);
                    let _ = tx.send(AppEvent::CaptureWindow { window_id }).await;
                }).unwrap();
            }
        });
    }

    // Show OCR window
    ocr_window.show()?;

    // Store current results for Anki card creation
    let results_store = std::rc::Rc::new(std::cell::RefCell::new(Vec::<DisplayResult>::new()));

    // Set up add-to-anki callback
    {
        let results_clone = results_store.clone();
        let tx = ui_to_app_tx.clone();
        window.on_add_to_anki(move |idx| {
            let results = results_clone.borrow();
            if let Some(result) = results.get(idx as usize) {
                let result = result.clone();
                let tx = tx.clone();
                slint::spawn_local(async move {
                    let _ = tx.send(AppEvent::CreateCard(result)).await;
                })
                .unwrap();
            }
        });
    }

    // Spawn a task to receive events from the app
    {
        slint::spawn_local(async move {
            while let Ok(event) = app_to_ui_rx.recv().await {
                if let Some(window) = window_weak.upgrade() {
                    match event {
                        AppEvent::UiEvent(UiEvent::Show) => {
                            window.show().ok();
                        }
                        AppEvent::UiEvent(UiEvent::Hide) => {
                            window.hide().ok();
                        }
                        AppEvent::UiEvent(UiEvent::Close) => {
                            window.hide().ok();
                            break;
                        }
                        AppEvent::ShowResults(results) => {
                            // Store results for Anki card creation
                            *results_store.borrow_mut() = results.clone();

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
                            window.set_results(model.into());
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        })
        .unwrap();
    }

    // Show the window
    window.show()?;

    // Run the Slint event loop
    window.run()?;

    Ok(())
}
