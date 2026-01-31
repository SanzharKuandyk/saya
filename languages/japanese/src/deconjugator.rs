use saya_core::language::DeconjugationResult;

pub struct JapaneseDeconjugator;

impl JapaneseDeconjugator {
    pub fn new() -> Self {
        Self
    }

    /// Deconjugate a word to its possible base forms
    pub fn deconjugate(&self, word: &str) -> Vec<DeconjugationResult> {
        let mut results = Vec::new();

        // Try て-form deconjugation
        results.extend(self.deconjugate_te_form(word));

        // Try た-form deconjugation
        results.extend(self.deconjugate_ta_form(word));

        // Try ます-form deconjugation
        results.extend(self.deconjugate_masu_form(word));

        // Try ている-form deconjugation
        results.extend(self.deconjugate_teiru_form(word));

        // Try negative forms
        results.extend(self.deconjugate_negative(word));

        // Try i-adjective conjugations
        results.extend(self.deconjugate_i_adjective(word));

        results
    }

    /// Deconjugate て-form verbs
    fn deconjugate_te_form(&self, word: &str) -> Vec<DeconjugationResult> {
        let mut results = Vec::new();

        if word.ends_with("て") {
            let stem = &word[..word.len() - 3]; // Remove て (3 bytes)

            // Godan verbs
            // いて → う (買って → 買う)
            if stem.ends_with("い") {
                let base = format!("{}う", &stem[..stem.len() - 3]);
                results.push(DeconjugationResult {
                    base_form: base,
                    conjugation_type: "godan verb, te-form".to_string(),
                    confidence: 0.7,
                });
            }
            // って → う/つ/る (待って → 待つ)
            if stem.ends_with("っ") {
                for ending in &["う", "つ", "る"] {
                    let base = format!("{}{}", &stem[..stem.len() - 3], ending);
                    results.push(DeconjugationResult {
                        base_form: base,
                        conjugation_type: "godan verb, te-form".to_string(),
                        confidence: 0.6,
                    });
                }
            }
            // んで → ぬ/ぶ/む (読んで → 読む)
            if stem.ends_with("ん") {
                for ending in &["ぬ", "ぶ", "む"] {
                    let base = format!("{}{}", &stem[..stem.len() - 3], ending);
                    results.push(DeconjugationResult {
                        base_form: base,
                        conjugation_type: "godan verb, te-form".to_string(),
                        confidence: 0.6,
                    });
                }
            }
            // いて → く (書いて → 書く)
            if stem.ends_with("い") {
                let base = format!("{}く", &stem[..stem.len() - 3]);
                results.push(DeconjugationResult {
                    base_form: base,
                    conjugation_type: "godan verb, te-form".to_string(),
                    confidence: 0.7,
                });
            }
            // して → す (話して → 話す)
            if stem.ends_with("し") {
                let base = format!("{}す", &stem[..stem.len() - 3]);
                results.push(DeconjugationResult {
                    base_form: base,
                    conjugation_type: "godan verb, te-form".to_string(),
                    confidence: 0.7,
                });
            }

            // Ichidan verbs (食べて → 食べる)
            let base = format!("{}る", stem);
            results.push(DeconjugationResult {
                base_form: base,
                conjugation_type: "ichidan verb, te-form".to_string(),
                confidence: 0.8,
            });
        }

        // Irregular: して → する
        if word == "して" {
            results.push(DeconjugationResult {
                base_form: "する".to_string(),
                conjugation_type: "irregular verb する, te-form".to_string(),
                confidence: 1.0,
            });
        }

        // Irregular: 来て → 来る
        if word == "来て" || word == "きて" {
            results.push(DeconjugationResult {
                base_form: "来る".to_string(),
                conjugation_type: "irregular verb 来る, te-form".to_string(),
                confidence: 1.0,
            });
        }

        results
    }

    /// Deconjugate た-form verbs
    fn deconjugate_ta_form(&self, word: &str) -> Vec<DeconjugationResult> {
        // Similar to て-form but with た instead of て
        if word.ends_with("た") {
            let te_form = format!("{}て", &word[..word.len() - 3]);
            return self.deconjugate_te_form(&te_form);
        }
        if word.ends_with("だ") {
            let te_form = format!("{}で", &word[..word.len() - 3]);
            return self.deconjugate_te_form(&te_form);
        }
        Vec::new()
    }

    /// Deconjugate ます-form verbs
    fn deconjugate_masu_form(&self, word: &str) -> Vec<DeconjugationResult> {
        let mut results = Vec::new();

        if word.ends_with("ます") {
            let stem = &word[..word.len() - 6]; // Remove ます (6 bytes)

            // Ichidan verbs (食べます → 食べる)
            let base = format!("{}る", stem);
            results.push(DeconjugationResult {
                base_form: base,
                conjugation_type: "ichidan verb, masu-form".to_string(),
                confidence: 0.8,
            });

            // Godan verbs - need to restore u-column
            // 書きます → 書く, 読みます → 読む, etc.
            for (i_sound, u_sound) in &[
                ("き", "く"),
                ("ぎ", "ぐ"),
                ("し", "す"),
                ("ち", "つ"),
                ("に", "ぬ"),
                ("び", "ぶ"),
                ("み", "む"),
                ("り", "る"),
            ] {
                if stem.ends_with(i_sound) {
                    let base_stem = &stem[..stem.len() - i_sound.len()];
                    let base = format!("{}{}", base_stem, u_sound);
                    results.push(DeconjugationResult {
                        base_form: base,
                        conjugation_type: "godan verb, masu-form".to_string(),
                        confidence: 0.8,
                    });
                }
            }
        }

        // します → する
        if word == "します" {
            results.push(DeconjugationResult {
                base_form: "する".to_string(),
                conjugation_type: "irregular verb する, masu-form".to_string(),
                confidence: 0.8,
            });
        }

        // 来ます → 来る
        if word == "来ます" || word == "きます" {
            results.push(DeconjugationResult {
                base_form: "来る".to_string(),
                conjugation_type: "irregular verb 来る, masu-form".to_string(),
                confidence: 0.8,
            });
        }

        results
    }

    /// Deconjugate ている-form verbs
    fn deconjugate_teiru_form(&self, word: &str) -> Vec<DeconjugationResult> {
        if word.ends_with("ている") {
            let te_form = format!("{}て", &word[..word.len() - 9]); // Remove いる
            return self
                .deconjugate_te_form(&te_form)
                .into_iter()
                .map(|mut r| {
                    r.conjugation_type = format!("{}, continuous", r.conjugation_type);
                    r
                })
                .collect();
        }
        Vec::new()
    }

    /// Deconjugate negative forms
    fn deconjugate_negative(&self, word: &str) -> Vec<DeconjugationResult> {
        let mut results = Vec::new();

        // ない-form (書かない → 書く)
        if word.ends_with("ない") {
            let stem = &word[..word.len() - 6]; // Remove ない

            // Godan verbs - a-column to u-column
            for (a_sound, u_sound) in &[
                ("か", "く"),
                ("が", "ぐ"),
                ("さ", "す"),
                ("た", "つ"),
                ("な", "ぬ"),
                ("ば", "ぶ"),
                ("ま", "む"),
                ("ら", "る"),
                ("わ", "う"),
            ] {
                if stem.ends_with(a_sound) {
                    let base_stem = &stem[..stem.len() - a_sound.len()];
                    let base = format!("{}{}", base_stem, u_sound);
                    results.push(DeconjugationResult {
                        base_form: base,
                        conjugation_type: "godan verb, negative".to_string(),
                        confidence: 0.8,
                    });
                }
            }

            // Ichidan verbs (食べない → 食べる)
            let base = format!("{}る", stem);
            results.push(DeconjugationResult {
                base_form: base,
                conjugation_type: "ichidan verb, negative".to_string(),
                confidence: 0.8,
            });
        }

        // しない → する
        if word == "しない" {
            results.push(DeconjugationResult {
                base_form: "する".to_string(),
                conjugation_type: "irregular verb する, negative".to_string(),
                confidence: 0.8,
            });
        }

        // 来ない → 来る
        if word == "来ない" || word == "こない" {
            results.push(DeconjugationResult {
                base_form: "来る".to_string(),
                conjugation_type: "irregular verb 来る, negative".to_string(),
                confidence: 0.8,
            });
        }

        results
    }

    /// Deconjugate i-adjective forms
    fn deconjugate_i_adjective(&self, word: &str) -> Vec<DeconjugationResult> {
        let mut results = Vec::new();

        // くない (negative): 高くない → 高い
        if word.ends_with("くない") {
            let stem = &word[..word.len() - 9]; // Remove くない
            let base = format!("{}い", stem);
            results.push(DeconjugationResult {
                base_form: base,
                conjugation_type: "i-adjective, negative".to_string(),
                confidence: 0.8,
            });
        }

        // かった (past): 高かった → 高い
        if word.ends_with("かった") {
            let stem = &word[..word.len() - 9]; // Remove かった
            let base = format!("{}い", stem);
            results.push(DeconjugationResult {
                base_form: base,
                conjugation_type: "i-adjective, past".to_string(),
                confidence: 0.8,
            });
        }

        // くて (te-form): 高くて → 高い
        if word.ends_with("くて") {
            let stem = &word[..word.len() - 6]; // Remove くて
            let base = format!("{}い", stem);
            results.push(DeconjugationResult {
                base_form: base,
                conjugation_type: "i-adjective, te-form".to_string(),
                confidence: 0.8,
            });
        }

        results
    }
}
