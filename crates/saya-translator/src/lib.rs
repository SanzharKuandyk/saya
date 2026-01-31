pub type LanguageCode = String;

/// Translation provider interface
#[async_trait::async_trait]
pub trait Translator: Send + Sync {
    /// Translate text from source to target language
    async fn translate(
        &self,
        text: &str,
        from: LanguageCode,
        to: LanguageCode,
    ) -> Result<Translation, TranslateError>;

    /// Detect language of text
    async fn detect_language(&self, text: &str) -> Result<LanguageCode, TranslateError>;

    /// Get supported language pairs
    fn supported_languages(&self) -> Vec<(LanguageCode, LanguageCode)>;

    /// Provider metadata
    fn metadata(&self) -> ProviderMetadata;
}

#[derive(Debug, Clone)]
pub struct Translation {
    pub text: String,
    pub from: LanguageCode,
    pub to: LanguageCode,
    pub provider: String,
    pub confidence: Option<f32>,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderMetadata {
    pub name: String,
    pub requires_api_key: bool,
    pub free_tier_available: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum TranslateError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Unsupported language pair: {from} -> {to}")]
    UnsupportedLanguagePair { from: String, to: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Authentication error")]
    AuthenticationError,
}
