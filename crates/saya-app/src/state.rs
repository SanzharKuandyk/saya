use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use saya_config::Config;
use tokio::sync::RwLock;
use windows::Media::Ocr::OcrEngine as WinOcrEngine;

pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub ocr_engine: WinOcrEngine,
    pub auto_ocr_running: AtomicBool,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let ocr_engine = saya_ocr::init_ocr_engine(&config.ocr.language).unwrap_or_else(|e| {
            tracing::error!("failed to initialize OCR engine: {:?}", e);
            panic!("Exiting due to OCR init failure");
        });

        Self {
            config: Arc::new(RwLock::new(config)),
            ocr_engine,
            auto_ocr_running: AtomicBool::new(false),
        }
    }
}
