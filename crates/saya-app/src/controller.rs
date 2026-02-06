use std::sync::Arc;
use std::time::Duration;

use kanal::{AsyncReceiver, AsyncSender};
use saya_lang_japanese::{JapaneseProcessor, JapaneseTranslator};
use saya_types::AppEvent;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::events::event_loop;
use crate::io::watcher_io;
use crate::state::AppState;
use crate::ui::ui_loop;

/// Centralized channel management
pub struct ChannelSet {
    pub app_to_ui: (AsyncSender<AppEvent>, AsyncReceiver<AppEvent>),
    pub ui_to_app: (AsyncSender<AppEvent>, AsyncReceiver<AppEvent>),
}

impl ChannelSet {
    pub fn new() -> Self {
        Self {
            app_to_ui: kanal::bounded_async(256),  // OCR burst capacity
            ui_to_app: kanal::bounded_async(64),   // UI interactions
        }
    }
}

/// Application controller for task spawning and lifecycle
pub struct AppController {
    channels: ChannelSet,
    state: Arc<AppState>,
    cancel_token: CancellationToken,
}

impl AppController {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            channels: ChannelSet::new(),
            state,
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn spawn_tasks(
        &self,
        processor: Arc<JapaneseProcessor>,
        translator: Arc<Option<JapaneseTranslator>>,
    ) -> JoinSet<anyhow::Result<()>> {
        let mut tasks = JoinSet::new();

        // Event loop
        tasks.spawn(event_loop(
            self.state.clone(),
            self.channels.ui_to_app.1.clone(),
            self.channels.app_to_ui.0.clone(),
            processor,
            translator,
        ));

        // UI loop
        tasks.spawn(ui_loop(
            self.channels.app_to_ui.1.clone(),
            self.channels.ui_to_app.0.clone(),
            self.state.config.clone(),
        ));

        // Watcher IO
        let watcher_interval = Duration::from_millis(100);
        tasks.spawn(watcher_io(
            self.state.clone(),
            watcher_interval,
            self.cancel_token.child_token(),
            self.channels.app_to_ui.0.clone(),
        ));

        tasks
    }

    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}
