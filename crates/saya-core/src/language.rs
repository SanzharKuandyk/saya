use std::collections::HashMap;

/// Text processing and lookup interface for language implementations
pub trait LanguageProcessor: Send + Sync {
    /// Language identifier (ISO 639-1 code: "ja", "zh", "ko", etc.)
    fn language_code(&self) -> &str;

    /// Normalize text (Unicode normalization, whitespace, etc.)
    fn normalize(&self, text: &str) -> String;

    /// Break text into processable tokens
    fn tokenize(&self, text: &str) -> Vec<Token>;

    /// Look up a token in the dictionary
    fn lookup(&self, token: &Token) -> Vec<LookupResult>;
}

/// Optional trait for languages with conjugation/declension
pub trait Deconjugator: Send + Sync {
    /// Convert conjugated form to dictionary form(s)
    fn deconjugate(&self, word: &str) -> Vec<DeconjugationResult>;
}

/// Optional trait for word frequency data
pub trait FrequencyProvider: Send + Sync {
    /// Get frequency rank (lower = more common), None if not in list
    fn frequency(&self, word: &str) -> Option<u32>;

    /// Get frequency percentile (0.0-100.0)
    fn percentile(&self, word: &str) -> Option<f32> {
        self.frequency(word).map(|rank| {
            100.0 - (rank as f32 / 100000.0).min(100.0)
        })
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub surface: String,
    pub normalized: String,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct LookupResult {
    pub term: String,
    pub readings: Vec<String>,
    pub definitions: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DeconjugationResult {
    pub base_form: String,
    pub conjugation_type: String,
    pub confidence: f32,
}
