mod ocr;
mod capture;
mod hotkey;

pub use ocr::recognize_sync;
pub use capture::{
    capture_screen_region, capture_primary_screen, capture_window,
    capture_window_by_title, list_windows, CaptureRegion,
};
pub use hotkey::HotkeyManager;
