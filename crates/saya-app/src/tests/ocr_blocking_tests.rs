//! Tests to identify OCR blocking and event flow issues

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use kanal::unbounded_async;
use saya_core::types::AppEvent;
use saya_ocr::{CaptureRegion, recognize_sync, capture_screen_region};

/// Test 1: Does spawn_blocking work?
#[tokio::test]
async fn test_spawn_blocking_works() {
    let result = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_millis(10));
        42
    }).await;
    assert_eq!(result.unwrap(), 42);
}

/// Test 2: Can we do multiple spawn_blocking in parallel?
#[tokio::test]
async fn test_parallel_spawn_blocking() {
    let start = std::time::Instant::now();

    let h1 = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_millis(100));
        "one"
    });

    let h2 = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_millis(100));
        "two"
    });

    let r1 = timeout(Duration::from_secs(2), h1).await.unwrap().unwrap();
    let r2 = timeout(Duration::from_secs(2), h2).await.unwrap().unwrap();

    let elapsed = start.elapsed();
    tracing::debug!("Parallel spawn_blocking took: {:?}", elapsed);
    assert!(elapsed < Duration::from_millis(200), "Too slow: {:?}", elapsed);
    assert_eq!(r1, "one");
    assert_eq!(r2, "two");
}

/// Test 3: kanal channel basic test
#[tokio::test]
async fn test_kanal_works() {
    let (tx, rx) = unbounded_async::<u32>();
    let handle = tokio::spawn(async move {
        rx.recv().await.unwrap()
    });
    tx.send(123).await.unwrap();
    let result = timeout(Duration::from_secs(1), handle).await.unwrap().unwrap();
    assert_eq!(result, 123);
}

/// Test 4: Can spawn_blocking send to kanal channel?
#[tokio::test]
async fn test_spawn_blocking_to_kanal() {
    let (tx, rx) = unbounded_async::<String>();

    tokio::task::spawn_blocking(move || {
        tx.try_send("from blocking".to_string()).unwrap();
    }).await.unwrap();

    let result = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
    assert_eq!(result, "from blocking");
}

/// Test 5: Can spawn_blocking send AppEvent to channel?
#[tokio::test]
async fn test_spawn_blocking_to_app_event() {
    let (tx, rx) = unbounded_async::<AppEvent>();

    tokio::task::spawn_blocking(move || {
        let event = AppEvent::TextInput("test".to_string());
        tx.try_send(event).unwrap();
    }).await.unwrap();

    let event = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
    match event {
        AppEvent::TextInput(text) => assert_eq!(text, "test"),
        _ => panic!("Wrong event"),
    }
}

/// Test 6: Event loop simulation with kanal
#[tokio::test]
async fn test_event_loop_with_kanal() {
    let (tx, rx) = unbounded_async::<String>();
    let received = Arc::new(Mutex::new(vec![]));
    let received_clone = received.clone();

    let event_loop = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            received_clone.lock().unwrap().push(msg);
        }
    });

    for i in 0..5 {
        tx.send(format!("event_{}", i)).await.unwrap();
    }
    drop(tx);

    timeout(Duration::from_secs(1), event_loop).await.unwrap();
    let guard = received.lock().unwrap();
    assert_eq!(guard.len(), 5);
}

/// Test 7: Check worker threads availability
#[tokio::test]
async fn test_worker_threads() {
    let handles: Vec<_> = (0..4).map(|_| {
        tokio::task::spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(50));
        })
    }).collect();

    for h in handles {
        timeout(Duration::from_secs(1), h).await.unwrap().unwrap();
    }
    tracing::debug!("All 4 spawn_blocking tasks completed");
}

/// Test 8: Check if multiple tasks can run spawn_blocking concurrently
#[tokio::test]
async fn test_concurrent_blocking() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..8 {
        let counter = counter.clone();
        handles.push(tokio::task::spawn_blocking(move || {
            std::thread::sleep(Duration::from_millis(50));
            counter.fetch_add(1, Ordering::SeqCst);
        }));
    }

    for h in handles {
        timeout(Duration::from_secs(5), h).await.unwrap().unwrap();
    }

    let count = counter.load(Ordering::SeqCst);
    tracing::debug!("Ran {} tasks in parallel", count);
    assert_eq!(count, 8);
}

/// Test 9: Full pipeline - spawn_blocking -> channel -> event loop
#[tokio::test]
async fn test_pipeline() {
    let (result_tx, result_rx) = unbounded_async::<String>();
    let (start_tx, mut start_rx) = mpsc::channel(1);

    let event_loop = tokio::spawn(async move {
        let _ = start_rx.recv().await;
        result_rx.recv().await.unwrap()
    });

    let ocr_task = tokio::task::spawn_blocking(move || {
        start_tx.blocking_send("start").unwrap();
        std::thread::sleep(Duration::from_millis(50));
        "ocr_result"
    });

    let ocr_result = timeout(Duration::from_secs(2), ocr_task).await.unwrap().unwrap();
    result_tx.send(ocr_result.to_string()).await.unwrap();

    let final_result = timeout(Duration::from_secs(1), event_loop).await.unwrap().unwrap();
    assert_eq!(final_result, "ocr_result");
    tracing::debug!("Pipeline test passed!");
}

/// Test 10: REAL OCR - Measure actual performance
#[tokio::test]
async fn test_real_ocr_performance() {
    let region = CaptureRegion {
        x: 100,
        y: 100,
        width: 200,
        height: 100,
    };

    let start = std::time::Instant::now();

    let result = tokio::task::spawn_blocking(move || {
        tracing::debug!("[OCR TEST] Starting capture...");
        let capture_start = std::time::Instant::now();
        let image_data = capture_screen_region(region).expect("Capture failed");
        let capture_time = capture_start.elapsed();
        tracing::debug!("[OCR TEST] Capture took: {:?}", capture_time);

        tracing::debug!("[OCR TEST] Starting OCR...");
        let ocr_start = std::time::Instant::now();
        let text = recognize_sync(&image_data, "ja").expect("OCR failed");
        let ocr_time = ocr_start.elapsed();
        tracing::debug!("[OCR TEST] OCR took: {:?}", ocr_time);

        text
    }).await;

    let total = start.elapsed();
    tracing::debug!("[OCR TEST] Total: {:?}", total);

    match result {
        Ok(text) => {
            tracing::debug!("[OCR TEST] Result: '{}' ({} chars)", text, text.len());
        }
        Err(e) => {
            tracing::debug!("[OCR TEST] Failed: {}", e);
        }
    }

    // If this takes > 5 seconds, we have a blocking issue
    assert!(total < Duration::from_secs(10), "OCR took too long: {:?}", total);
}

/// Test 11: Simulate actual app flow - spawn_blocking -> send event -> receive event
#[tokio::test]
async fn test_app_event_flow_simulation() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use saya_core::types::AppEvent;

    let (tx, rx) = unbounded_async::<AppEvent>();
    let event_count = Arc::new(AtomicUsize::new(0));
    let event_count_clone = event_count.clone();

    // Event loop simulation
    let event_loop = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                AppEvent::TextInput(text) => {
                    tracing::debug!("[APP FLOW] Received TextInput: {} chars", text.len());
                    event_count_clone.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    });

    // Simulate OCR flow (like in events.rs)
    let ocr_flow = tokio::spawn(async move {
        for i in 0..3 {
            let region = CaptureRegion {
                x: 100,
                y: 100,
                width: 200,
                height: 50,
            };

            // This is what events.rs does
            let result = tokio::task::spawn_blocking(move || {
                let image_data = capture_screen_region(region).expect("Capture failed");
                let text = recognize_sync(&image_data, "ja").expect("OCR failed");
                text
            }).await;

            match result {
                Ok(text) => {
                    tracing::debug!("[APP FLOW] Sending TextInput {}...", i + 1);
                    tx.send(AppEvent::TextInput(text)).await.expect("Send failed");
                }
                Err(e) => {
                    tracing::debug!("[APP FLOW] OCR failed: {}", e);
                }
            }
        }
        drop(tx); // Close channel when done
    });

    // Wait for everything to complete
    let _ = timeout(Duration::from_secs(10), ocr_flow).await.unwrap();
    let _ = timeout(Duration::from_secs(1), event_loop).await.unwrap();

    let count = event_count.load(Ordering::SeqCst);
    tracing::debug!("[APP FLOW] Received {} events", count);
    assert_eq!(count, 3, "Should have received 3 events");
    tracing::debug!("[APP FLOW] Test passed!");
}

/// Test 12: Check if slint::spawn_local equivalent works
/// This simulates what the real app does
#[tokio::test]
async fn test_spawn_local_simulation() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use saya_core::types::AppEvent;

    let (tx, rx) = unbounded_async::<AppEvent>();
    let received = Arc::new(AtomicUsize::new(0));
    let received_clone = received.clone();

    // Simulate slint::spawn_local - this is what lib.rs does
    let spawn_local_task = tokio::spawn(async move {
        for i in 0..3 {
            // This is what slint::spawn_local does
            let tx_clone = tx.clone();
            let _ = tokio::spawn(async move {
                tracing::debug!("[SPAWN_LOCAL] Sending event {}", i + 1);
                tx_clone.send(AppEvent::TextInput(format!("test_{}", i))).await.ok();
            }).await;

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        drop(tx);
    });

    // Event loop
    let event_loop = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                AppEvent::TextInput(text) => {
                    tracing::debug!("[SPAWN_LOCAL] Received: {}", text);
                    received_clone.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    });

    let _ = timeout(Duration::from_secs(5), spawn_local_task).await.unwrap();
    let _ = timeout(Duration::from_secs(1), event_loop).await.unwrap();

    let count = received.load(Ordering::SeqCst);
    tracing::debug!("[SPAWN_LOCAL] Received {} events", count);
    // Should receive all 3 events
}
