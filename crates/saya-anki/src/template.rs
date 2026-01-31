use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardTemplate {
    pub deck: String,
    pub model: String,
    pub front_template: String,
    pub back_template: String,
}

impl CardTemplate {
    /// Create default Japanese vocabulary template
    pub fn default_japanese() -> Self {
        Self {
            deck: "Japanese".to_string(),
            model: "Basic".to_string(),
            front_template: "{term}\n{reading}".to_string(),
            back_template: "{definition}".to_string(),
        }
    }

    /// Create custom template
    pub fn new(deck: String, model: String, front: String, back: String) -> Self {
        Self {
            deck,
            model,
            front_template: front,
            back_template: back,
        }
    }

    /// Format the front of the card
    pub fn format_front(&self, term: &str, reading: &str, definition: &str) -> String {
        self.front_template
            .replace("{term}", term)
            .replace("{reading}", reading)
            .replace("{definition}", definition)
    }

    /// Format the back of the card
    pub fn format_back(&self, term: &str, reading: &str, definition: &str) -> String {
        self.back_template
            .replace("{term}", term)
            .replace("{reading}", reading)
            .replace("{definition}", definition)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteField {
    pub name: String,
    pub value: String,
}
