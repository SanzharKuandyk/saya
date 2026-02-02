use anyhow::{Context, Result};
use windows::{
    core::HSTRING,
    Globalization::Language,
    Graphics::Imaging::{BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat},
    Media::Ocr::OcrEngine as WinOcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
};

/// Perform OCR on PNG/BMP image bytes
pub fn recognize_sync(image_bytes: &[u8], language_code: &str) -> Result<String> {
    println!(
        ">>> [OCR] recognize_sync: {} bytes, lang={}",
        image_bytes.len(),
        language_code
    );

    // Create OCR engine
    let language = Language::CreateLanguage(&HSTRING::from(language_code))
        .with_context(|| format!("Invalid language code: {}", language_code))?;

    let engine = WinOcrEngine::TryCreateFromLanguage(&language).with_context(|| {
        format!(
            "OCR engine failed for '{}'. Is the language pack installed?",
            language_code
        )
    })?;

    println!(">>> [OCR] Engine created");

    // Load image into memory stream
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(image_bytes)?;
    writer.StoreAsync()?.get()?;
    writer.FlushAsync()?.get()?;
    stream.Seek(0)?;

    println!(">>> [OCR] Image loaded to stream");

    // Decode image
    let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;

    // Convert to Bgra8 format required by OCR
    let bitmap = decoder
        .GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Bgra8, BitmapAlphaMode::Premultiplied)?
        .get()?;

    println!(
        ">>> [OCR] Bitmap: {}x{}",
        bitmap.PixelWidth()?,
        bitmap.PixelHeight()?
    );

    // Run OCR
    let result = engine.RecognizeAsync(&bitmap)?.get()?;
    let text = result.Text()?.to_string();

    println!(">>> [OCR] Result: '{}' ({} chars)", text, text.len());

    Ok(text)
}
