use anyhow::{Context, Result};
use windows::{
    core::HSTRING,
    Globalization::Language,
    Graphics::Imaging::BitmapDecoder,
    Media::Ocr::OcrEngine as WinOcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
};

pub struct OcrEngine {
    engine: WinOcrEngine,
}

impl OcrEngine {
    /// Create a new OCR engine for the specified language (e.g., "ja", "en")
    pub fn new(language_code: &str) -> Result<Self> {
        let language = Language::CreateLanguage(&HSTRING::from(language_code))
            .context("Failed to create language")?;

        let engine = WinOcrEngine::TryCreateFromLanguage(&language)
            .context("Failed to create OCR engine for language")?;

        Ok(Self { engine })
    }

    /// Recognize text from PNG image bytes
    pub async fn recognize(&self, image_bytes: &[u8]) -> Result<String> {
        // Create in-memory stream from image bytes
        let stream = InMemoryRandomAccessStream::new().context("Failed to create stream")?;
        let writer = DataWriter::CreateDataWriter(&stream).context("Failed to create writer")?;

        writer
            .WriteBytes(image_bytes)
            .context("Failed to write image bytes")?;
        writer
            .StoreAsync()
            .context("Failed to store async")?
            .get()
            .context("Failed to store data")?;
        writer.FlushAsync().context("Failed to flush")?.get()?;

        // Seek to beginning
        stream.Seek(0).context("Failed to seek")?;

        // Decode image to SoftwareBitmap
        let decoder = BitmapDecoder::CreateAsync(&stream)
            .context("Failed to create decoder async")?
            .get()
            .context("Failed to get decoder")?;

        let bitmap = decoder
            .GetSoftwareBitmapAsync()
            .context("Failed to get bitmap async")?
            .get()
            .context("Failed to get software bitmap")?;

        // Perform OCR
        let result = self
            .engine
            .RecognizeAsync(&bitmap)
            .context("Failed to recognize async")?
            .get()
            .context("Failed to get OCR result")?;

        // Extract text from result
        Ok(result.Text().context("Failed to get text")?.to_string())
    }

    /// Get the recognizer language for this engine
    pub fn recognizer_language(&self) -> Result<String> {
        self.engine
            .RecognizerLanguage()
            .context("Failed to get recognizer language")?
            .LanguageTag()
            .map(|tag| tag.to_string())
            .context("Failed to get language tag")
    }
}
