use async_trait::async_trait;
use saya_translator::{LanguageCode, ProviderMetadata, TranslateError, Translation, Translator};

#[derive(Clone)]
pub struct JapaneseTranslator {
    client: reqwest::Client,
    api_key: String,
    api_url: String,
}

impl JapaneseTranslator {
    pub fn new(api_key: String, api_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            api_url,
        }
    }
}

#[async_trait]
impl Translator for JapaneseTranslator {
    async fn translate(
        &self,
        text: &str,
        from: LanguageCode,
        to: LanguageCode,
    ) -> Result<Translation, TranslateError> {
        if self.api_key.is_empty() {
            return Err(TranslateError::AuthenticationError);
        }

        let params = [
            ("text", text),
            ("source_lang", &from.to_uppercase()),
            ("target_lang", &to.to_uppercase()),
        ];

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .form(&params)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(TranslateError::RateLimitExceeded);
        }

        if response.status() == 403 {
            return Err(TranslateError::AuthenticationError);
        }

        if !response.status().is_success() {
            return Err(TranslateError::ApiError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            TranslateError::ApiError(format!("Failed to parse response: {}", e))
        })?;

        let translated_text = json["translations"]
            .get(0)
            .and_then(|t| t["text"].as_str())
            .ok_or_else(|| TranslateError::ApiError("No translation in response".to_string()))?;

        Ok(Translation {
            text: translated_text.to_string(),
            from,
            to,
            provider: "deepl".to_string(),
            confidence: None,
            alternatives: vec![],
        })
    }

    async fn detect_language(&self, text: &str) -> Result<LanguageCode, TranslateError> {
        if self.api_key.is_empty() {
            return Err(TranslateError::AuthenticationError);
        }

        let params = [("text", text), ("target_lang", "EN")];

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(TranslateError::ApiError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            TranslateError::ApiError(format!("Failed to parse response: {}", e))
        })?;

        let detected = json["translations"]
            .get(0)
            .and_then(|t| t["detected_source_language"].as_str())
            .ok_or_else(|| TranslateError::ApiError("No detected language".to_string()))?;

        Ok(detected.to_lowercase())
    }

    fn supported_languages(&self) -> Vec<(LanguageCode, LanguageCode)> {
        vec![
            ("ja".to_string(), "en".to_string()),
            ("ja".to_string(), "de".to_string()),
            ("ja".to_string(), "fr".to_string()),
            ("ja".to_string(), "es".to_string()),
            ("ja".to_string(), "zh".to_string()),
            ("en".to_string(), "ja".to_string()),
        ]
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "DeepL".to_string(),
            requires_api_key: true,
            free_tier_available: true,
        }
    }
}
