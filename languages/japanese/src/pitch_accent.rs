use std::collections::HashMap;

/// Japanese pitch accent provider
pub struct JapanesePitchAccent {
    accents: HashMap<String, PitchPattern>,
}

impl JapanesePitchAccent {
    /// Create empty pitch accent database
    pub fn new() -> Self {
        Self {
            accents: HashMap::new(),
        }
    }

    /// Create with some common word patterns
    pub fn with_defaults() -> Self {
        let mut accents = HashMap::new();

        // Common words with pitch accent patterns
        // Format: (word, drop position) - 0 = heiban (flat), 1+ = odaka/atamadaka/nakadaka
        let patterns = [
            ("日本", 0),  // heiban
            ("東京", 0),  // heiban
            ("学校", 0),  // heiban
            ("先生", 3),  // odaka
            ("学生", 0),  // heiban
            ("時間", 0),  // heiban
            ("本", 1),    // atamadaka
            ("水", 0),    // heiban
            ("山", 0),    // heiban
            ("川", 0),    // heiban
        ];

        for (word, drop) in patterns {
            accents.insert(word.to_string(), PitchPattern::from_drop_position(drop));
        }

        Self { accents }
    }

    /// Load pitch accent data from TSV file (word\treading\tdrop_position format)
    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut accents = HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                if let Ok(drop) = parts[1].parse::<u8>() {
                    accents.insert(
                        parts[0].to_string(),
                        PitchPattern::from_drop_position(drop),
                    );
                }
            }
        }

        Ok(Self { accents })
    }

    /// Get pitch accent pattern for a word
    pub fn get_pattern(&self, word: &str) -> Option<&PitchPattern> {
        self.accents.get(word)
    }

    /// Get pitch accent notation string
    pub fn get_notation(&self, word: &str) -> Option<String> {
        self.get_pattern(word).map(|p| p.to_notation())
    }
}

#[derive(Debug, Clone)]
pub struct PitchPattern {
    /// Drop position (0 = heiban, 1 = atamadaka, 2+ = nakadaka/odaka)
    pub drop_position: u8,
    /// Pattern type
    pub pattern_type: PatternType,
}

impl PitchPattern {
    /// Create pattern from drop position
    pub fn from_drop_position(drop: u8) -> Self {
        let pattern_type = match drop {
            0 => PatternType::Heiban,   // 平板型 (flat)
            1 => PatternType::Atamadaka, // 頭高型 (head-high)
            _ => PatternType::Nakadaka,  // 中高型/尾高型 (mid-high/tail-high)
        };

        Self {
            drop_position: drop,
            pattern_type,
        }
    }

    /// Convert to notation string
    pub fn to_notation(&self) -> String {
        match self.pattern_type {
            PatternType::Heiban => "◎".to_string(),     // Flat
            PatternType::Atamadaka => "①".to_string(),  // Drop after 1st mora
            PatternType::Nakadaka => format!("⓪{}", self.drop_position), // Drop at position
            PatternType::Odaka => "⓪".to_string(),      // Drop at end
        }
    }

    /// Get pattern type name
    pub fn type_name(&self) -> &'static str {
        self.pattern_type.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    Heiban,     // 平板型 - flat (no drop)
    Atamadaka,  // 頭高型 - head-high (drop after 1st mora)
    Nakadaka,   // 中高型 - mid-high (drop in middle)
    Odaka,      // 尾高型 - tail-high (drop at end)
}

impl PatternType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PatternType::Heiban => "Heiban (Flat)",
            PatternType::Atamadaka => "Atamadaka (Head-high)",
            PatternType::Nakadaka => "Nakadaka (Mid-high)",
            PatternType::Odaka => "Odaka (Tail-high)",
        }
    }
}
