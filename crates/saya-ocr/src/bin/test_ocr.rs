//! Simple OCR test - run with: cargo run -p saya-ocr --bin test_ocr

use anyhow::Result;

fn main() -> Result<()> {
    tracing::debug!("=== OCR Test ===\n");

    // 1. List available windows
    tracing::debug!("1. Available windows:");
    let windows = saya_ocr::list_windows()?;
    for (i, (id, title)) in windows.iter().enumerate().take(10) {
        tracing::debug!("   [{}] {} - {}", i, id, title);
    }

    // 2. Capture primary screen
    tracing::debug!("\n2. Capturing primary screen...");
    let start = std::time::Instant::now();
    let png_data = saya_ocr::capture_primary_screen()?;
    tracing::debug!("   {} bytes in {:?}", png_data.len(), start.elapsed());

    // 3. Save for inspection
    std::fs::write("test_capture.png", &png_data)?;
    tracing::debug!("   Saved to test_capture.png");

    // 4. Run OCR
    tracing::debug!("\n3. Running OCR (ja)...");
    let start = std::time::Instant::now();
    match saya_ocr::recognize_sync(&png_data, "ja") {
        Ok(text) => {
            tracing::debug!("   {:?} - {} chars", start.elapsed(), text.len());
            if !text.is_empty() {
                for line in text.lines().take(5) {
                    tracing::debug!("   > {}", line);
                }
            }
        }
        Err(e) => tracing::debug!("   Failed: {}", e),
    }

    // 5. Test window capture if there's a window
    if let Some((id, title)) = windows.first() {
        tracing::debug!("\n4. Testing window capture: '{}'", title);
        match saya_ocr::capture_window(*id) {
            Ok(data) => tracing::debug!("   Captured {} bytes", data.len()),
            Err(e) => tracing::debug!("   Failed: {}", e),
        }
    }

    tracing::debug!("\n=== Done ===");
    Ok(())
}
