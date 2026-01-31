use saya_core::dictionary::{Dictionary, DictionaryEntry, DictionaryMetadata, Definition, SearchOptions};

/// JMdict dictionary entry
#[derive(Debug, Clone)]
pub struct JMdictEntry {
    pub id: String,
    pub kanji: Vec<String>,
    pub readings: Vec<String>,
    pub meanings: Vec<String>,
    pub pos: Vec<String>,
    pub jlpt_level: Option<u8>,
    pub frequency_rank: Option<u32>,
}

impl DictionaryEntry for JMdictEntry {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn headword(&self) -> String {
        self.kanji.first()
            .or_else(|| self.readings.first())
            .cloned()
            .unwrap_or_default()
    }

    fn readings(&self) -> Vec<String> {
        self.readings.clone()
    }

    fn definitions(&self) -> Vec<Definition> {
        self.meanings.iter().map(|text| Definition {
            text: text.clone(),
            part_of_speech: self.pos.clone(),
            tags: vec![],
        }).collect()
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "kanji": self.kanji,
            "jlpt_level": self.jlpt_level,
            "frequency_rank": self.frequency_rank,
        })
    }
}

/// JMdict dictionary
pub struct JMdict {
    entries: Vec<JMdictEntry>,
}

impl JMdict {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl Dictionary for JMdict {
    fn lookup_exact(&self, query: &str) -> Vec<Box<dyn DictionaryEntry>> {
        self.entries.iter()
            .filter(|e| e.kanji.iter().any(|k| k == query) || e.readings.iter().any(|r| r == query))
            .map(|e| Box::new(e.clone()) as Box<dyn DictionaryEntry>)
            .collect()
    }

    fn search(&self, query: &str, _options: SearchOptions) -> Vec<Box<dyn DictionaryEntry>> {
        self.lookup_exact(query)
    }

    fn get_by_id(&self, id: &str) -> Option<Box<dyn DictionaryEntry>> {
        self.entries.iter()
            .find(|e| e.id == id)
            .map(|e| Box::new(e.clone()) as Box<dyn DictionaryEntry>)
    }

    fn metadata(&self) -> DictionaryMetadata {
        DictionaryMetadata {
            name: "JMdict".to_string(),
            version: "1.0".to_string(),
            language: "ja".to_string(),
            entry_count: self.entries.len(),
        }
    }
}
