use std::collections::HashMap;

/// Japanese word frequency provider
pub struct JapaneseFrequency {
    frequencies: HashMap<String, u32>,
}

impl JapaneseFrequency {
    /// Create empty frequency database
    pub fn new() -> Self {
        Self {
            frequencies: HashMap::new(),
        }
    }

    /// Create with embedded high-frequency words
    pub fn with_defaults() -> Self {
        let mut frequencies = HashMap::new();

        // Top 100 most common Japanese words with approximate rankings
        // Data based on Japanese frequency corpus (simplified)
        let common_words = [
            ("の", 1), ("に", 2), ("は", 3), ("を", 4), ("た", 5),
            ("が", 6), ("で", 7), ("て", 8), ("と", 9), ("し", 10),
            ("れ", 11), ("さ", 12), ("ある", 13), ("いる", 14), ("も", 15),
            ("する", 16), ("から", 17), ("な", 18), ("こ", 19), ("として", 20),
            ("い", 21), ("や", 22), ("れる", 23), ("など", 24), ("なっ", 25),
            ("ない", 26), ("この", 27), ("ため", 28), ("その", 29), ("あっ", 30),
            ("よう", 31), ("また", 32), ("もの", 33), ("という", 34), ("あり", 35),
            ("まで", 36), ("られ", 37), ("なる", 38), ("へ", 39), ("か", 40),
            ("だ", 41), ("これ", 42), ("によって", 43), ("により", 44), ("おり", 45),
            ("より", 46), ("による", 47), ("ず", 48), ("なり", 49), ("られる", 50),
            ("において", 51), ("ば", 52), ("なかっ", 53), ("なく", 54), ("しかし", 55),
            ("について", 56), ("せ", 57), ("だっ", 58), ("その後", 59), ("できる", 60),
            ("それ", 61), ("う", 62), ("ので", 63), ("なお", 64), ("のみ", 65),
            ("でき", 66), ("日本", 67), ("思う", 68), ("それぞれ", 69), ("とき", 70),
            ("ほか", 71), ("行う", 72), ("考える", 73), ("示す", 74), ("用いる", 75),
            ("言う", 76), ("大きい", 77), ("多い", 78), ("新しい", 79), ("良い", 80),
            ("高い", 81), ("長い", 82), ("強い", 83), ("少ない", 84), ("古い", 85),
            ("見る", 86), ("来る", 87), ("持つ", 88), ("使う", 89), ("出る", 90),
            ("取る", 91), ("分かる", 92), ("行く", 93), ("入る", 94), ("作る", 95),
            ("聞く", 96), ("話す", 97), ("読む", 98), ("書く", 99), ("食べる", 100),
        ];

        for (word, rank) in common_words {
            frequencies.insert(word.to_string(), rank);
        }

        Self { frequencies }
    }

    /// Load frequency data from TSV file (word\trank format)
    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut frequencies = HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                if let Ok(rank) = parts[1].parse::<u32>() {
                    frequencies.insert(parts[0].to_string(), rank);
                }
            }
        }

        Ok(Self { frequencies })
    }

    /// Get frequency rank for a word (lower = more common)
    pub fn get_rank(&self, word: &str) -> Option<u32> {
        self.frequencies.get(word).copied()
    }

    /// Get frequency level (Common, Uncommon, Rare)
    pub fn get_level(&self, word: &str) -> FrequencyLevel {
        match self.get_rank(word) {
            Some(rank) if rank <= 1000 => FrequencyLevel::VeryCommon,
            Some(rank) if rank <= 5000 => FrequencyLevel::Common,
            Some(rank) if rank <= 10000 => FrequencyLevel::Uncommon,
            Some(_) => FrequencyLevel::Rare,
            None => FrequencyLevel::Unknown,
        }
    }

    /// Get star rating (1-5 stars based on frequency)
    pub fn get_stars(&self, word: &str) -> u8 {
        match self.get_rank(word) {
            Some(rank) if rank <= 500 => 5,
            Some(rank) if rank <= 2000 => 4,
            Some(rank) if rank <= 5000 => 3,
            Some(rank) if rank <= 10000 => 2,
            Some(_) => 1,
            None => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyLevel {
    VeryCommon,
    Common,
    Uncommon,
    Rare,
    Unknown,
}

impl FrequencyLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            FrequencyLevel::VeryCommon => "Very Common",
            FrequencyLevel::Common => "Common",
            FrequencyLevel::Uncommon => "Uncommon",
            FrequencyLevel::Rare => "Rare",
            FrequencyLevel::Unknown => "Unknown",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            FrequencyLevel::VeryCommon => "★★★★★",
            FrequencyLevel::Common => "★★★★",
            FrequencyLevel::Uncommon => "★★★",
            FrequencyLevel::Rare => "★★",
            FrequencyLevel::Unknown => "",
        }
    }
}
