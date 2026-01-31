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
