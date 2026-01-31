use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct AnkiConnectClient {
    base_url: String,
    client: reqwest::Client,
}

impl AnkiConnectClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Check if AnkiConnect is available
    pub async fn check_connection(&self) -> Result<u32> {
        let response: AnkiResponse<u32> = self.invoke("version", json!({})).await?;
        response.into_result()
    }

    /// Get list of deck names
    pub async fn deck_names(&self) -> Result<Vec<String>> {
        let response: AnkiResponse<Vec<String>> = self.invoke("deckNames", json!({})).await?;
        response.into_result()
    }

    /// Get list of model (note type) names
    pub async fn model_names(&self) -> Result<Vec<String>> {
        let response: AnkiResponse<Vec<String>> = self.invoke("modelNames", json!({})).await?;
        response.into_result()
    }

    /// Add a note to Anki
    pub async fn add_note(
        &self,
        deck: &str,
        model: &str,
        front: &str,
        back: &str,
    ) -> Result<u64> {
        let params = json!({
            "note": {
                "deckName": deck,
                "modelName": model,
                "fields": {
                    "Front": front,
                    "Back": back
                },
                "tags": ["saya"]
            }
        });

        let response: AnkiResponse<u64> = self.invoke("addNote", params).await?;
        response.into_result()
    }

    /// Invoke an AnkiConnect API action
    async fn invoke<T>(&self, action: &str, params: serde_json::Value) -> Result<AnkiResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let request = AnkiRequest {
            action: action.to_string(),
            version: 6,
            params,
        };

        let response = self
            .client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to AnkiConnect")?;

        response
            .json::<AnkiResponse<T>>()
            .await
            .context("Failed to parse AnkiConnect response")
    }
}

#[derive(Serialize)]
struct AnkiRequest {
    action: String,
    version: u32,
    params: serde_json::Value,
}

#[derive(Deserialize)]
struct AnkiResponse<T> {
    result: Option<T>,
    error: Option<String>,
}

impl<T> AnkiResponse<T> {
    fn into_result(self) -> Result<T> {
        if let Some(error) = self.error {
            anyhow::bail!("AnkiConnect error: {}", error);
        }

        self.result
            .context("AnkiConnect returned null result")
    }
}
