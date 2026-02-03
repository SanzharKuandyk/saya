use anyhow::Context;
use windows::{
    Globalization::Language,
    Graphics::Imaging::{BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat},
    Media::Ocr::OcrEngine as WinOcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
    core::HSTRING,
};

/// Wrapper around async func called via tokio::spawn_blocking
pub fn recognize_sync(image_bytes: &[u8], language_code: &str) -> anyhow::Result<String> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(recognize_async(image_bytes, language_code))
    })
}

/// Perform OCR on PNG/BMP image bytes
pub async fn recognize_async(image_bytes: &[u8], language_code: &str) -> anyhow::Result<String> {
    tracing::debug!(
        ">>> [OCR] recognize_async: {} bytes, lang={}",
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

    tracing::debug!(">>> [OCR] Engine created");

    // Load image into memory stream
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(image_bytes)?;
    writer.StoreAsync()?.await?;
    writer.FlushAsync()?.await?;
    stream.Seek(0)?;

    tracing::debug!(">>> [OCR] Image loaded to stream");

    // Decode image
    let decoder = BitmapDecoder::CreateAsync(&stream)?.await?;

    // Convert to Bgra8 format required by OCR
    let bitmap = decoder
        .GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Bgra8, BitmapAlphaMode::Premultiplied)?
        .await?;

    tracing::debug!(
        ">>> [OCR] Bitmap: {}x{}",
        bitmap.PixelWidth()?,
        bitmap.PixelHeight()?
    );

    // Run OCR
    let result = engine.RecognizeAsync(&bitmap)?.await?;
    let text = result.Text()?.to_string();

    tracing::debug!(">>> [OCR] Result: '{}' ({} chars)", text, text.len());

    Ok(text)
}
