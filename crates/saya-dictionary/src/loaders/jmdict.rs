use std::path::Path;

use crate::types::{DictEntry, Dictionary};

pub struct JMdict {
    entries: Vec<DictEntry>,
}

impl Dictionary for JMdict {
    fn name(&self) -> &str {
        "JMdict"
    }
    fn description(&self) -> &str {
        "Japanese-English dictionary"
    }

    fn lookup(&self, term: &str) -> Vec<DictEntry> {
        self.entries
            .iter()
            .filter(|e| e.kanji.iter().any(|k| k == term) || e.readings.iter().any(|r| r == term))
            .cloned()
            .collect()
    }

    fn load(&mut self, path: &Path) -> Result<(), String> {
        // parse XML/JSON into self.entries
        Ok(())
    }
}
