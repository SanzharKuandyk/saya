use std::time::Duration;

use arboard::Clipboard;
use tokio::time;

pub async fn watch_clipboard<F>(mut on_text: F) -> Result<(), anyhow::Error>
where
    F: FnMut(String) + Send + 'static,
{
    let mut clipboard = Clipboard::new()?;
    let mut last_text = String::new();

    let mut interval = time::interval(Duration::from_millis(500));

    loop {
        interval.tick().await;
        if let Ok(text) = clipboard.get_text()
            && !text.is_empty()
            && text != last_text
        {
            last_text = text.clone();
            on_text(text);
        }
    }
}
