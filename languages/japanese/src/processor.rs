use saya_core::language::{LanguageProcessor, Token, LookupResult};
use unicode_normalization::UnicodeNormalization;

use crate::deconjugator::JapaneseDeconjugator;
use crate::dictionary::JMdict;

/// Japanese language processor
pub struct JapaneseProcessor {
    dictionary: JMdict,
    deconjugator: JapaneseDeconjugator,
}

impl JapaneseProcessor {
    pub fn new(dictionary: JMdict) -> Self {
        Self {
            dictionary,
            deconjugator: JapaneseDeconjugator::new(),
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

                    // Add conjugation info to the result
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

        results
    }
}
