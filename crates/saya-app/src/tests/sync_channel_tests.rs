use saya_core::types::{AppEvent, CaptureRegion};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_tokio_spawn_from_sync_context() {
    let (tx, rx) = kanal::unbounded_async::<AppEvent>();

    let sync_callback = move || {
        tracing::debug!("Sync callback: spawning tokio task");
        let tx = tx.clone();
        tokio::spawn(async move {
            tracing::debug!("Tokio task: sending event");
            tx.send(AppEvent::TextInput("test".to_string()))
                .await
                .expect("send failed");
            tracing::debug!("Tokio task: event sent");
        });
        tracing::debug!("Sync callback: returned immediately");
    };

    sync_callback();
    tracing::debug!("Async test: waiting for event");

    let result = timeout(Duration::from_secs(2), rx.recv()).await;

    match result {
        Ok(Ok(AppEvent::TextInput(text))) => {
            tracing::debug!("Event received successfully");
            assert_eq!(text, "test");
        }
        Ok(Ok(_)) => panic!("Wrong event type"),
        Ok(Err(e)) => panic!("Channel error: {}", e),
        Err(_) => panic!("Timeout - tokio::spawn from sync context failed!"),
    }
}

#[tokio::test]
async fn test_ui_button_click_with_tokio_spawn() {
    let (tx, rx) = kanal::unbounded_async::<AppEvent>();

    let button_click = move || {
        tracing::debug!("Button click (sync context)");
        let tx = tx.clone();
        tokio::spawn(async move {
            tx.send(AppEvent::TriggerOcr(CaptureRegion {
                x: 100,
                y: 200,
                width: 300,
                height: 400,
            }))
            .await
            .expect("send failed");
        });
    };

    button_click();
    tracing::debug!("Button click returned immediately");

    let result = timeout(Duration::from_secs(2), rx.recv()).await;

    match result {
        Ok(Ok(AppEvent::TriggerOcr(CaptureRegion {
            x,
            y,
            width,
            height,
        }))) => {
            tracing::debug!("Event received");
            assert_eq!(x, 100);
            assert_eq!(y, 200);
            assert_eq!(width, 300);
            assert_eq!(height, 400);
        }
        Ok(Ok(_)) => panic!("Wrong event type"),
        Ok(Err(e)) => panic!("Channel error: {}", e),
        Err(_) => panic!("Timeout - event never arrived!"),
    }
}

#[tokio::test]
async fn test_multiple_spawned_sends() {
    let (tx, rx) = kanal::unbounded_async::<AppEvent>();

    for i in 0..100 {
        let tx = tx.clone();
        tokio::spawn(async move {
            tx.send(AppEvent::TextInput(format!("msg{}", i)))
                .await
                .expect("send failed");
        });
    }

    let mut count = 0;
    let result = timeout(Duration::from_secs(2), async {
        while count < 100 {
            rx.recv().await.expect("recv failed");
            count += 1;
        }
    })
    .await;

    assert!(result.is_ok(), "Timeout waiting for events!");
    assert_eq!(count, 100);
}
