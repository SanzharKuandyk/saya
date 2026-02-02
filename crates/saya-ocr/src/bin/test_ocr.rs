//! Simple OCR test - run with: cargo run -p saya-ocr --bin test_ocr

use anyhow::Result;

fn main() -> Result<()> {
    println!("=== OCR Test ===\n");

    // 1. List available windows
    println!("1. Available windows:");
    let windows = saya_ocr::list_windows()?;
    for (i, (id, title)) in windows.iter().enumerate().take(10) {
        println!("   [{}] {} - {}", i, id, title);
    }

    // 2. Capture primary screen
    println!("\n2. Capturing primary screen...");
    let start = std::time::Instant::now();
    let png_data = saya_ocr::capture_primary_screen()?;
    println!("   {} bytes in {:?}", png_data.len(), start.elapsed());

    // 3. Save for inspection
    std::fs::write("test_capture.png", &png_data)?;
    println!("   Saved to test_capture.png");

    // 4. Run OCR
    println!("\n3. Running OCR (ja)...");
    let start = std::time::Instant::now();
    match saya_ocr::recognize_sync(&png_data, "ja") {
        Ok(text) => {
            println!("   {:?} - {} chars", start.elapsed(), text.len());
            if !text.is_empty() {
                for line in text.lines().take(5) {
                    println!("   > {}", line);
                }
            }
        }
        Err(e) => println!("   Failed: {}", e),
    }

    // 5. Test window capture if there's a window
    if let Some((id, title)) = windows.first() {
        println!("\n4. Testing window capture: '{}'", title);
        match saya_ocr::capture_window(*id) {
            Ok(data) => println!("   Captured {} bytes", data.len()),
            Err(e) => println!("   Failed: {}", e),
        }
    }

    println!("\n=== Done ===");
    Ok(())
}
