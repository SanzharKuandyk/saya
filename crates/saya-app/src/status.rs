use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// OCR status information
#[derive(Clone, Debug, Default)]
pub struct OcrStatus {
    pub capturing: bool,
    pub last_capture_time: Option<SystemTime>,
    pub capture_count: u64,
    pub error_count: u64,
    pub current_message: String,
}

/// Application status
pub struct AppStatus {
    pub ocr: Arc<RwLock<OcrStatus>>,
}

impl AppStatus {
    pub fn new() -> Self {
        Self {
            ocr: Arc::new(RwLock::new(OcrStatus::default())),
        }
    }
}

impl Default for AppStatus {
    fn default() -> Self {
        Self::new()
    }
}
