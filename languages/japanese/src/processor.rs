use std::path::Path;
use saya_core::language::{LanguageProcessor, Token, LookupResult};
use unicode_normalization::UnicodeNormalization;

use crate::deconjugator::JapaneseDeconjugator;
use crate::dictionary::JMdict;
use crate::frequency::JapaneseFrequency;
use crate::jlpt::JlptLevels;
use crate::loader::JMdictLoader;
use crate::pitch_accent::JapanesePitchAccent;

/// Japanese language processor
pub struct JapaneseProcessor {
    dictionary: JMdict,
    deconjugator: JapaneseDeconjugator,
    frequency: JapaneseFrequency,
    pitch_accent: JapanesePitchAccent,
    jlpt: JlptLevels,
}

impl JapaneseProcessor {
    /// Create a new Japanese processor with default configuration (embedded dictionary)
    pub fn new() -> Self {
        Self::with_additional_dicts(&[])
    }

    /// Create a new Japanese processor with additional dictionary paths
    pub fn with_additional_dicts(additional_paths: &[String]) -> Self {
        // Load embedded dictionary
        let mut dict = JMdictLoader::load_embedded()
            .unwrap_or_else(|e| {
                tracing::error!("Failed to load embedded dictionary: {}", e);
                tracing::warn!("Starting with empty dictionary");
                JMdict::new()
            });

        // Load and merge additional dictionaries
        for path in additional_paths {
            match JMdictLoader::load_from_file(Path::new(path)) {
                Ok(additional) => {
                    tracing::info!("Merging additional dictionary from: {}", path);
                    dict = JMdictLoader::merge(dict, additional);
                }
                Err(e) => {
                    tracing::warn!("Failed to load dictionary from {}: {}", path, e);
                }
            }
        }

        Self {
            dictionary: dict,
            deconjugator: JapaneseDeconjugator::new(),
            frequency: JapaneseFrequency::with_defaults(),
            pitch_accent: JapanesePitchAccent::with_defaults(),
            jlpt: JlptLevels::with_defaults(),
        }
    }
}

impl LanguageProcessor for JapaneseProcessor {
    fn language_code(&self) -> &str {
        "ja"
    }

    fn normalize(&self, text: &str) -> String {
        text.nfkc()
            .collect::<String>()
            .chars()
            .filter(|c| !c.is_whitespace() || *c == ' ')
            .collect()
    }

    fn tokenize(&self, text: &str) -> Vec<Token> {
        let normalized = self.normalize(text);
        let chars: Vec<char> = normalized.chars().collect();
        let mut tokens = Vec::new();

        for i in 0..chars.len() {
            for len in (1..=chars.len().saturating_sub(i).min(10)).rev() {
                let surface: String = chars[i..i + len].iter().collect();
                tokens.push(Token {
                    surface: surface.clone(),
                    normalized: surface,
                    position: i,
                });
            }
        }

        tokens
    }

    fn lookup(&self, token: &Token) -> Vec<LookupResult> {
        use saya_core::dictionary::Dictionary;

        // Try direct lookup first
        let mut results: Vec<LookupResult> = self
            .dictionary
            .lookup_exact(&token.normalized)
            .into_iter()
            .map(|entry| entry.to_lookup_result())
            .collect();

        // If direct lookup failed, try deconjugation
        if results.is_empty() {
            let deconj_results = self.deconjugator.deconjugate(&token.normalized);

            for deconj in deconj_results {
                let base_results = self.dictionary.lookup_exact(&deconj.base_form);

                for entry in base_results {
                    let mut result = entry.to_lookup_result();

                    // Add conjugation info
                    result.metadata.insert(
                        "conjugation".to_string(),
                        format!(
                            "{} → {} ({})",
                            token.normalized, deconj.base_form, deconj.conjugation_type
                        ),
                    );
                    result.metadata.insert(
                        "base_form".to_string(),
                        deconj.base_form.clone(),
                    );

                    results.push(result);
                }
            }
        }

        // Add frequency, pitch accent, and JLPT data to all results
        for result in &mut results {
            let term = &result.term;

            // Frequency data
            if let Some(rank) = self.frequency.get_rank(term) {
                result.metadata.insert("frequency_rank".to_string(), rank.to_string());
            }
            let level = self.frequency.get_level(term);
            result.metadata.insert("frequency_level".to_string(), level.as_str().to_string());
            let stars = self.frequency.get_stars(term);
            if stars > 0 {
                result.metadata.insert("frequency_stars".to_string(), "★".repeat(stars as usize));
            }

            // Pitch accent
            if let Some(notation) = self.pitch_accent.get_notation(term) {
                result.metadata.insert("pitch_accent".to_string(), notation);
            }

            // JLPT level
            if let Some(badge) = self.jlpt.get_badge(term) {
                result.metadata.insert("jlpt_level".to_string(), badge);
            }
        }

        results
    }
}
