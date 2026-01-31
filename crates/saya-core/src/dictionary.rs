use std::collections::HashMap;
use std::path::Path;

use crate::language::LookupResult;

/// Dictionary lookup operations
pub trait Dictionary: Send + Sync {
    /// Search dictionary by exact match
    fn lookup_exact(&self, query: &str) -> Vec<Box<dyn DictionaryEntry>>;

    /// Search dictionary with options
    fn search(&self, query: &str, options: SearchOptions) -> Vec<Box<dyn DictionaryEntry>>;

    /// Get entry by unique ID if supported
    fn get_by_id(&self, id: &str) -> Option<Box<dyn DictionaryEntry>>;

    /// Get dictionary metadata
    fn metadata(&self) -> DictionaryMetadata;
}

/// Individual dictionary entry
pub trait DictionaryEntry: Send + Sync {
    /// Unique entry ID
    fn id(&self) -> String;

    /// Main headword/term
    fn headword(&self) -> String;

    /// All possible readings/pronunciations
    fn readings(&self) -> Vec<String>;

    /// All definitions
    fn definitions(&self) -> Vec<Definition>;

    /// Language-specific data as JSON
    fn metadata(&self) -> serde_json::Value;

    /// Convert to generic lookup result
    fn to_lookup_result(&self) -> LookupResult {
        LookupResult {
            term: self.headword(),
            readings: self.readings(),
            definitions: self.definitions().iter().map(|d| d.text.clone()).collect(),
            metadata: HashMap::new(),
        }
    }
}

/// Load dictionaries from files or embedded data
pub trait DictionaryLoader {
    /// Load dictionary from file path
    fn load_from_file(&self, path: &Path) -> Result<Box<dyn Dictionary>, LoadError>;

    /// Supported file formats
    fn supported_formats(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub max_results: usize,
    pub match_type: MatchType,
    pub language_specific: HashMap<String, String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_results: 10,
            match_type: MatchType::Exact,
            language_specific: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MatchType {
    Exact,
    Prefix,
    Suffix,
    Contains,
}

#[derive(Debug, Clone)]
pub struct DictionaryMetadata {
    pub name: String,
    pub version: String,
    pub language: String,
    pub entry_count: usize,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub text: String,
    pub part_of_speech: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
