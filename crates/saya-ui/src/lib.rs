use std::sync::{Arc, Mutex};

use events::handle_events;
use kanal::{AsyncReceiver, AsyncSender, Receiver, Sender};
use saya_config::Config;
use saya_types::{AppEvent, CaptureRegion, DisplayResult};
use tokio::sync::RwLock;

pub mod bridge;
pub mod events;
pub mod state;

slint::include_modules!();

/// Parse a hex color string (#RRGGBB or #RRGGBBAA) into a Slint Color
fn parse_color(hex: &str) -> Result<slint::Color, String> {
    let hex = hex.trim_start_matches('#');

    let (r, g, b, a) = match hex.len() {
        6 => {
            // #RRGGBB format
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
            (r, g, b, 255u8)
        }
        8 => {
            // #RRGGBBAA format
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
            let a = u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?;
            (r, g, b, a)
        }
        _ => return Err(format!("Invalid color format: #{}", hex)),
    };

    Ok(slint::Color::from_argb_u8(a, r, g, b))
}

pub async fn ui_loop(
    app_to_ui_rx: AsyncReceiver<AppEvent>,
    ui_to_app_tx: AsyncSender<AppEvent>,
    config: Arc<RwLock<Config>>,
) -> anyhow::Result<()> {
    tracing::info!("UI loop starting");

    let (sync_tx, sync_rx) = kanal::unbounded::<AppEvent>();
    let (app_sync_tx, app_sync_rx) = kanal::unbounded::<AppEvent>();

    let config = config.read().await.clone();
    let ui_thread = std::thread::spawn(move || run_slint_ui(sync_tx, app_sync_rx, &config));

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
    config: &Config,
) -> anyhow::Result<()> {
    tracing::info!("[SLINT] UI thread starting");

    let window = OverlayWindow::new()?;
    let window_weak = window.as_weak();

    let ocr_window = OcrWindow::new()?;
    let ocr_window_weak = ocr_window.as_weak();
    let ocr_auto = config.ocr.auto;

    ocr_window.set_auto_capturing_mode(ocr_auto);
    window.set_ocr_auto_mode(ocr_auto);

    // Set border colors from config
    if let Ok(color) = parse_color(&config.ocr.border_ready_color) {
        ocr_window.set_border_ready_color(color);
    }
    if let Ok(color) = parse_color(&config.ocr.border_capturing_color) {
        ocr_window.set_border_capturing_color(color);
    }
    if let Ok(color) = parse_color(&config.ocr.border_preparing_color) {
        ocr_window.set_border_preparing_color(color);
    }

    tracing::debug!("[SLINT] UI windows created");

    let window_ids = std::rc::Rc::new(std::cell::RefCell::new(Vec::<u32>::new()));

    // Timer to update capture region when window moves (for auto OCR)
    {
        let ocr_weak = ocr_window.as_weak();
        let tx = ui_to_app_tx.clone();

        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(500),
            move || {
                if let Some(win) = ocr_weak.upgrade() {
                    // Only send updates if auto mode is enabled
                    if win.get_auto_capturing_mode() {
                        let pos = win.window().position();
                        let size = win.window().size();

                        let header_height = 32i32;
                        let capture_height = size.height.saturating_sub(32);

                        let region = CaptureRegion {
                            x: pos.x,
                            y: pos.y + header_height,
                            width: size.width,
                            height: capture_height,
                        };

                        let _ = tx.send(AppEvent::UpdateCaptureRegion(region));
                    }
                }
            },
        );
    }

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

    // Window close handler
    {
        let ocr_weak = ocr_window.as_weak();

        ocr_window.on_window_closed(move || {
            if let Some(win) = ocr_weak.upgrade() {
                win.window().hide().ok();
                tracing::info!("[SLINT] OCR window closed");
            }
        });
    }

    // Window resize handler
    {
        let ocr_weak = ocr_window.as_weak();
        let tx = ui_to_app_tx.clone();

        ocr_window.on_window_resized(move || {
            if let Some(win) = ocr_weak.upgrade() {
                let pos = win.window().position();
                let size = win.window().size();

                let header_height = 32i32;
                let capture_height = size.height.saturating_sub(32);

                let region = CaptureRegion {
                    x: pos.x,
                    y: pos.y + header_height,
                    width: size.width,
                    height: capture_height,
                };

                tracing::debug!("[SLINT] Window resized, updating region: {:?}", region);
                let _ = tx.send(AppEvent::UpdateCaptureRegion(region));
            }
        });
    }

    // OCR auto mode toggle (from main window)
    {
        let window_weak_clone = window_weak.clone();
        let ocr_weak = ocr_window.as_weak();
        let tx = ui_to_app_tx.clone();
        let toggling = std::rc::Rc::new(std::cell::Cell::new(false));

        window.on_toggle_ocr_auto(move || {
            // Prevent rapid clicking
            if toggling.get() {
                return;
            }
            toggling.set(true);

            if let Some(win) = window_weak_clone.upgrade() {
                let new_mode = !win.get_ocr_auto_mode();
                win.set_ocr_auto_mode(new_mode);
                tracing::info!("[SLINT] Auto mode toggled to: {}", new_mode);

                // Sync to OCR window
                if let Some(ocr_win) = ocr_weak.upgrade() {
                    ocr_win.set_auto_capturing_mode(new_mode);

                    // If enabling auto mode, trigger auto OCR with current region
                    if new_mode {
                        let pos = ocr_win.window().position();
                        let size = ocr_win.window().size();

                        let header_height = 32i32;
                        let capture_height = size.height.saturating_sub(32);

                        let region = CaptureRegion {
                            x: pos.x,
                            y: pos.y + header_height,
                            width: size.width,
                            height: capture_height,
                        };

                        let _ = tx.send(AppEvent::TriggerAutoOcr(region));
                    }
                }
            }

            // Reset toggle flag after a short delay
            let toggling_clone = toggling.clone();
            slint::Timer::single_shot(std::time::Duration::from_millis(100), move || {
                toggling_clone.set(false);
            });
        });
    }

    // OCR capture/stop button (from main window)
    {
        let window_weak_clone = window_weak.clone();
        let ocr_weak = ocr_window.as_weak();
        let tx = ui_to_app_tx.clone();

        window.on_trigger_ocr_capture(move || {
            if let Some(win) = window_weak_clone.upgrade() {
                if win.get_ocr_auto_mode() {
                    // Stop auto mode
                    win.set_ocr_auto_mode(false);
                    if let Some(ocr_win) = ocr_weak.upgrade() {
                        ocr_win.set_auto_capturing_mode(false);
                    }
                    tracing::info!("[SLINT] Auto mode stopped");
                } else {
                    // Trigger single capture
                    if let Some(ocr_win) = ocr_weak.upgrade() {
                        let pos = ocr_win.window().position();
                        let size = ocr_win.window().size();

                        let header_height = 32i32;
                        let capture_height = size.height.saturating_sub(32);

                        let region = CaptureRegion {
                            x: pos.x,
                            y: pos.y + header_height,
                            width: size.width,
                            height: capture_height,
                        };

                        tracing::info!("[SLINT] Manual capture triggered");
                        let _ = tx.send(AppEvent::TriggerOcr(region));
                    }
                }
            }
        });
    }

    {
        let tx = ui_to_app_tx.clone();
        let ocr_weak = ocr_window.as_weak();
        let ids = window_ids.clone();

        ocr_window.on_capture_clicked(move || {
            tracing::debug!("[SLINT] OCR capture button clicked");

            if let Some(win) = ocr_weak.upgrade() {
                win.set_is_capturing(true);
                win.set_status("".into());

                let pos = win.window().position();
                let size = win.window().size();

                // Calculate capture zone (exclude header 32px)
                let header_height = 32i32;
                let capture_height = size.height.saturating_sub(32);

                let selected_idx = win.get_selected_window_index();
                let window_id = if selected_idx >= 0 {
                    ids.borrow().get(selected_idx as usize).copied()
                } else {
                    None
                };

                tracing::info!(
                    "[SLINT] Capturing region: {}x{} at ({}, {}), window: {:?}",
                    size.width,
                    capture_height,
                    pos.x,
                    pos.y + header_height,
                    window_id
                );

                // Always send with region coordinates
                let region = CaptureRegion {
                    x: pos.x,
                    y: pos.y + header_height,
                    width: size.width,
                    height: capture_height,
                };

                let _ = send_capture_region(region, tx.clone(), ocr_auto);
            }
        });
    }

    // Auto-populate window list on startup (but don't select any)
    if let Ok(windows) = saya_ocr::list_windows() {
        let mut stored_ids = window_ids.borrow_mut();
        let titles: Vec<slint::SharedString> = windows
            .iter()
            .map(|(id, title)| {
                stored_ids.push(*id);
                title.chars().take(40).collect::<String>().into()
            })
            .collect();

        let model = std::rc::Rc::new(slint::VecModel::from(titles));
        ocr_window.set_window_list(model.into());
        tracing::debug!("[SLINT] Auto-populated {} windows", stored_ids.len());
    }

    ocr_window.show()?;
    tracing::debug!("[SLINT] OCR window shown");

    let results_store = Arc::new(Mutex::new(Vec::<DisplayResult>::new()));

    // Show config overlay handler
    {
        let window_weak_clone = window_weak.clone();
        window.on_show_config(move || {
            if let Some(win) = window_weak_clone.upgrade() {
                win.set_config_visible(true);
                tracing::info!("[SLINT] Config overlay opened");
            }
        });
    }

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

                let _ = slint::invoke_from_event_loop(move || {
                    handle_events(event, window_weak, ocr_weak, &results_store);
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

pub fn send_capture_region(
    region: CaptureRegion,
    tx: Sender<AppEvent>,
    auto: bool,
) -> anyhow::Result<()> {
    let event = if auto {
        AppEvent::TriggerAutoOcr(region)
    } else {
        AppEvent::TriggerOcr(region)
    };
    match tx.send(event) {
        Ok(_) => tracing::info!("[SLINT] Capture Region is sent"),
        Err(e) => tracing::error!("[SLINT] Send failed: {}", e),
    }

    Ok(())
}
