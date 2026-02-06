use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use saya_types::DisplayResult;

/// UI-specific state (separate from AppState)
pub struct UiState {
    pub results: Arc<Mutex<Vec<DisplayResult>>>,
    pub window_ids: Rc<RefCell<Vec<u32>>>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            window_ids: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
