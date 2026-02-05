mod capture;
mod com;
mod hotkey;
mod ocr;

pub use capture::{
    capture_primary_screen, capture_screen_region, capture_window, capture_window_by_title,
    list_windows,
};
pub use com::ComGuard;
pub use hotkey::HotkeyManager;
pub use ocr::{init_ocr_engine, recognize_sync};
