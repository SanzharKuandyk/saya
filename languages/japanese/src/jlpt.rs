use std::collections::HashMap;

/// JLPT level provider
pub struct JlptLevels {
    levels: HashMap<String, JlptLevel>,
}

impl JlptLevels {
    /// Create empty JLPT database
    pub fn new() -> Self {
        Self {
            levels: HashMap::new(),
        }
    }

    /// Create with some common words
    pub fn with_defaults() -> Self {
        let mut levels = HashMap::new();

        // N5 (beginner) - ~800 words
        let n5_words = [
            "の", "に", "は", "を", "です", "ます", "でした", "ました",
            "日本", "人", "本", "先生", "学生", "学校", "時間", "今", "明日", "昨日",
            "食べる", "飲む", "見る", "聞く", "話す", "読む", "書く", "行く", "来る",
            "大きい", "小さい", "高い", "安い", "良い", "悪い", "新しい", "古い",
            "一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
        ];

        // N4 (elementary) - ~1500 words
        let n4_words = [
            "考える", "思う", "分かる", "知る", "教える", "習う", "始める", "終わる",
            "働く", "勉強", "仕事", "会社", "時計", "電話", "手紙", "映画",
            "強い", "弱い", "優しい", "厳しい", "美しい", "汚い", "便利", "不便",
        ];

        // N3 (intermediate) - ~3750 words
        let n3_words = [
            "経験", "研究", "発見", "意見", "説明", "計画", "準備", "確認",
            "複雑", "簡単", "正確", "曖昧", "適切", "不適切", "重要", "軽視",
        ];

        for word in n5_words {
            levels.insert(word.to_string(), JlptLevel::N5);
        }

        for word in n4_words {
            levels.insert(word.to_string(), JlptLevel::N4);
        }

        for word in n3_words {
            levels.insert(word.to_string(), JlptLevel::N3);
        }

        Self { levels }
    }

    /// Load JLPT levels from TSV file (word\tlevel format)
    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut levels = HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                if let Some(level) = JlptLevel::from_str(parts[1]) {
                    levels.insert(parts[0].to_string(), level);
                }
            }
        }

        Ok(Self { levels })
    }

    /// Get JLPT level for a word
    pub fn get_level(&self, word: &str) -> Option<JlptLevel> {
        self.levels.get(word).copied()
    }

    /// Get level string
    pub fn get_level_str(&self, word: &str) -> Option<&'static str> {
        self.get_level(word).map(|l| l.as_str())
    }

    /// Get level badge
    pub fn get_badge(&self, word: &str) -> Option<String> {
        self.get_level(word).map(|l| l.badge())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JlptLevel {
    N5, // Beginner (~800 words)
    N4, // Elementary (~1500 words)
    N3, // Intermediate (~3750 words)
    N2, // Upper intermediate (~6000 words)
    N1, // Advanced (~10000 words)
}

impl JlptLevel {
    /// Parse level from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "N5" => Some(JlptLevel::N5),
            "N4" => Some(JlptLevel::N4),
            "N3" => Some(JlptLevel::N3),
            "N2" => Some(JlptLevel::N2),
            "N1" => Some(JlptLevel::N1),
            _ => None,
        }
    }

    /// Get level string
    pub fn as_str(&self) -> &'static str {
        match self {
            JlptLevel::N5 => "N5",
            JlptLevel::N4 => "N4",
            JlptLevel::N3 => "N3",
            JlptLevel::N2 => "N2",
            JlptLevel::N1 => "N1",
        }
    }

    /// Get level description
    pub fn description(&self) -> &'static str {
        match self {
            JlptLevel::N5 => "N5 (Beginner)",
            JlptLevel::N4 => "N4 (Elementary)",
            JlptLevel::N3 => "N3 (Intermediate)",
            JlptLevel::N2 => "N2 (Upper Intermediate)",
            JlptLevel::N1 => "N1 (Advanced)",
        }
    }

    /// Get color badge
    pub fn badge(&self) -> String {
        match self {
            JlptLevel::N5 => "🟢 N5".to_string(),
            JlptLevel::N4 => "🟡 N4".to_string(),
            JlptLevel::N3 => "🟠 N3".to_string(),
            JlptLevel::N2 => "🔴 N2".to_string(),
            JlptLevel::N1 => "🟣 N1".to_string(),
        }
    }
}
