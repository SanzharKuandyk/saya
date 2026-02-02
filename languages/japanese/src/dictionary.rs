use std::collections::HashMap;
use saya_core::dictionary::{Dictionary, DictionaryEntry, DictionaryMetadata, Definition, SearchOptions};
use serde::Deserialize;

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

// JSON structures for parsing jmdict-simplified format
#[derive(Debug, Deserialize)]
struct JMdictJson {
    words: Vec<JMdictJsonEntry>,
}

#[derive(Debug, Deserialize)]
struct JMdictJsonEntry {
    id: String,
    #[serde(default)]
    kanji: Vec<KanjiElement>,
    #[serde(default)]
    kana: Vec<KanaElement>,
    sense: Vec<Sense>,
}

#[derive(Debug, Deserialize)]
struct KanjiElement {
    text: String,
}

#[derive(Debug, Deserialize)]
struct KanaElement {
    text: String,
}

#[derive(Debug, Deserialize)]
struct Sense {
    #[serde(rename = "partOfSpeech", default)]
    part_of_speech: Vec<String>,
    gloss: Vec<Gloss>,
}

#[derive(Debug, Deserialize)]
struct Gloss {
    lang: String,
    text: String,
}

/// JMdict dictionary
pub struct JMdict {
    entries: Vec<JMdictEntry>,
    kanji_index: HashMap<String, Vec<usize>>,
    reading_index: HashMap<String, Vec<usize>>,
}

impl JMdict {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            kanji_index: HashMap::new(),
            reading_index: HashMap::new(),
        }
    }

    /// Load JMdict from JSON string (jmdict-simplified format)
    pub fn from_json(json_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data: JMdictJson = serde_json::from_str(json_str)?;

        let mut entries = Vec::new();
        let mut kanji_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut reading_index: HashMap<String, Vec<usize>> = HashMap::new();

        for json_entry in data.words {
            // Extract kanji and readings
            let kanji: Vec<String> = json_entry.kanji.iter().map(|k| k.text.clone()).collect();
            let readings: Vec<String> = json_entry.kana.iter().map(|k| k.text.clone()).collect();

            // Extract English meanings and POS
            let mut meanings = Vec::new();
            let mut pos = Vec::new();

            for sense in &json_entry.sense {
                // Only use English glosses
                for gloss in &sense.gloss {
                    if gloss.lang == "eng" {
                        meanings.push(gloss.text.clone());
                    }
                }
                // Collect POS tags
                pos.extend(sense.part_of_speech.clone());
            }

            // Skip entries with no English meanings
            if meanings.is_empty() {
                continue;
            }

            let entry = JMdictEntry {
                id: json_entry.id,
                kanji: kanji.clone(),
                readings: readings.clone(),
                meanings,
                pos,
                jlpt_level: None,
                frequency_rank: None,
            };

            let entry_idx = entries.len();
            entries.push(entry);

            // Build indices
            for k in kanji {
                kanji_index.entry(k).or_insert_with(Vec::new).push(entry_idx);
            }
            for r in readings {
                reading_index.entry(r).or_insert_with(Vec::new).push(entry_idx);
            }
        }

        Ok(Self {
            entries,
            kanji_index,
            reading_index,
        })
    }

    /// Get the number of entries in the dictionary
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Merge another dictionary into this one
    /// Entries from the other dictionary with the same ID will override existing ones
    pub fn merge(mut self, other: JMdict) -> Self {
        use std::collections::HashSet;

        // Collect IDs from base dictionary
        let mut existing_ids: HashSet<String> = self.entries.iter().map(|e| e.id.clone()).collect();

        // Add entries from other dictionary
        for entry in other.entries.into_iter() {
            if existing_ids.contains(&entry.id) {
                // Override: remove old entry and add new one
                self.entries.retain(|e| e.id != entry.id);
            }
            existing_ids.insert(entry.id.clone());

            let entry_idx = self.entries.len();
            self.entries.push(entry.clone());

            // Update indices
            for k in &entry.kanji {
                self.kanji_index.entry(k.clone()).or_insert_with(Vec::new).push(entry_idx);
            }
            for r in &entry.readings {
                self.reading_index.entry(r.clone()).or_insert_with(Vec::new).push(entry_idx);
            }
        }

        self
    }
}

impl Dictionary for JMdict {
    fn lookup_exact(&self, query: &str) -> Vec<Box<dyn DictionaryEntry>> {
        let mut result_indices: Vec<usize> = Vec::new();

        // Check kanji index
        if let Some(indices) = self.kanji_index.get(query) {
            result_indices.extend(indices);
        }

        // Check reading index
        if let Some(indices) = self.reading_index.get(query) {
            result_indices.extend(indices);
        }

        // Deduplicate and collect entries
        result_indices.sort_unstable();
        result_indices.dedup();

        result_indices
            .into_iter()
            .filter_map(|idx| self.entries.get(idx))
            .map(|e: &JMdictEntry| Box::new(e.clone()) as Box<dyn DictionaryEntry>)
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
