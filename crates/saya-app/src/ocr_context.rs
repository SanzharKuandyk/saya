use std::sync::Arc;

use kanal::AsyncSender;
use saya_lang_japanese::{JapaneseProcessor, JapaneseTranslator};
use saya_types::AppEvent;

use crate::AppState;

/// Encapsulates OCR dependencies to reduce parameter passing
///
/// This context struct bundles all the shared dependencies needed for OCR operations,
/// reducing function signatures from 6 parameters to 3 (context, region, auto flag).
pub struct OcrContext {
    pub state: Arc<AppState>,
    pub event_tx: AsyncSender<AppEvent>,
    pub processor: Arc<JapaneseProcessor>,
    pub translator: Arc<Option<JapaneseTranslator>>,
}

impl OcrContext {
    pub fn new(
        state: Arc<AppState>,
        event_tx: AsyncSender<AppEvent>,
        processor: Arc<JapaneseProcessor>,
        translator: Arc<Option<JapaneseTranslator>>,
    ) -> Self {
        Self {
            state,
            event_tx,
            processor,
            translator,
        }
    }

    /// Clone the context for passing to async tasks
    ///
    /// This clones all Arc references, incrementing reference counts
    /// but not duplicating the underlying data.
    pub fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            event_tx: self.event_tx.clone(),
            processor: self.processor.clone(),
            translator: self.translator.clone(),
        }
    }
}
