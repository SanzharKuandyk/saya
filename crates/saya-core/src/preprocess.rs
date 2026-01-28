use unicode_normalization::UnicodeNormalization;

pub trait Preprocessor {
    // Default JP preprocessor
    fn process(&self, text: &str) -> String {
        let mut text = text.trim().to_string();

        if text.is_empty() {
            return text;
        }

        // Unicode normalization (NFKC)
        text = text.nfkc().collect();

        // Optional: remove extra whitespace/newlines
        text = text.replace(['\n', '\r'], "").trim().to_string();

        text
    }
}

pub struct DefaultPreprocessor;
impl Preprocessor for DefaultPreprocessor {}
