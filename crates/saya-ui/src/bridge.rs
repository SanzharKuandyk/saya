use kanal::{AsyncReceiver, AsyncSender, Receiver, Sender};
use saya_types::AppEvent;

/// Bridge between async backend and sync UI thread
pub struct UiBridge {
    to_ui_tx: Sender<AppEvent>,
    from_ui_rx: AsyncReceiver<AppEvent>,
}

pub struct UiBridgeHandle {
    pub to_ui_rx: Receiver<AppEvent>,
    pub from_ui_tx: AsyncSender<AppEvent>,
}

impl UiBridge {
    pub fn new() -> (Self, UiBridgeHandle) {
        let (to_ui_tx, to_ui_rx) = kanal::bounded(128);
        let (from_ui_tx, from_ui_rx) = kanal::bounded_async(64);

        (
            UiBridge { to_ui_tx, from_ui_rx },
            UiBridgeHandle { to_ui_rx, from_ui_tx },
        )
    }

    pub async fn forward_from_backend(&self, app_to_ui_rx: AsyncReceiver<AppEvent>) {
        while let Ok(event) = app_to_ui_rx.recv().await {
            if self.to_ui_tx.send(event).is_err() {
                break;
            }
        }
    }

    pub async fn forward_to_backend(&self, ui_to_app_tx: AsyncSender<AppEvent>) {
        while let Ok(event) = self.from_ui_rx.recv().await {
            if ui_to_app_tx.send(event).await.is_err() {
                break;
            }
        }
    }
}
