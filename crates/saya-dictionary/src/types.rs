use std::path::Path;

#[derive(Debug, Clone)]
pub struct DictEntry {
    pub kanji: Vec<String>,           // e.g., ["食べる"]
    pub readings: Vec<String>,        // e.g., ["たべる"]
    pub meanings: Vec<String>,        // e.g., ["to eat"]
    pub pos: Vec<String>,             // e.g., ["verb"]
    pub jlpt_level: Option<u8>,       // e.g., Some(5)
    pub frequency_rank: Option<u32>,  // optional usage frequency
    pub pitch_accent: Option<String>, // e.g., "LH"
}

pub struct DictInfo {
    pub name: String,
    pub language: String, // "ja-en"
    pub entry_count: usize,
}

pub trait Dictionary {
    fn name(&self) -> &str; // e.g., "JMdict"
    fn description(&self) -> &str; // optional description
    fn lookup(&self, term: &str) -> Vec<DictEntry>; // return matching entries
    fn load(&mut self, path: &Path) -> Result<(), String>; // load from file
}
