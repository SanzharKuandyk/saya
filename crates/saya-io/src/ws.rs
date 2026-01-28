use futures_util::StreamExt;
use tokio_tungstenite::connect_async;

pub async fn start_ws_listener<F>(url: &str, mut on_text: F) -> Result<(), anyhow::Error>
where
    F: FnMut(String) + Send + 'static,
{
    let (ws_stream, _) = connect_async(url).await?;
    let (_, mut read) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(msg) = msg
                && msg.is_text()
            {
                on_text(msg.to_text().unwrap().to_string());
            }
        }
    });

    Ok(())
}
