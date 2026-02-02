use std::path::Path;
use crate::dictionary::JMdict;

pub struct JMdictLoader;

impl JMdictLoader {
    /// Load embedded dictionary data
    pub fn load_embedded() -> Result<JMdict, Box<dyn std::error::Error>> {
        let json = include_str!("../data/jmdict_eng.json");
        tracing::info!("Loading embedded JMdict dictionary...");
        let dict = JMdict::from_json(json)?;
        tracing::info!("Loaded {} dictionary entries", dict.entry_count());
        Ok(dict)
    }

    /// Load dictionary from file path
    pub fn load_from_file(path: &Path) -> Result<JMdict, Box<dyn std::error::Error>> {
        tracing::info!("Loading JMdict from file: {}", path.display());
        let json = std::fs::read_to_string(path)?;
        let dict = JMdict::from_json(&json)?;
        tracing::info!("Loaded {} dictionary entries from file", dict.entry_count());
        Ok(dict)
    }

    /// Merge two dictionaries (later entries override earlier ones by ID)
    pub fn merge(base: JMdict, additional: JMdict) -> JMdict {
        base.merge(additional)
    }
}
