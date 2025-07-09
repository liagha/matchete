use {
    crate::{
        assessor::{Resembler, Resemblance},
        prelude::string::utils::edit_distance,
    },
    core::cmp::max,
    hashish::{HashSet},
};

#[derive(PartialEq)]
pub struct Tokens {
    separators: Vec<char>,
}

impl Default for Tokens {
    fn default() -> Self {
        Self { separators: vec!['_', '-', '.', ' '] }
    }
}

impl Tokens {
    pub fn new(separators: Vec<char>) -> Self {
        Self { separators }
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        for c in text.chars() {
            if self.separators.contains(&c) {
                if !current.is_empty() {
                    tokens.push(current);
                    current = String::new();
                }
            } else {
                if !current.is_empty() && current.chars().last().map_or(false, |last| !last.is_uppercase() && c.is_uppercase()) {
                    tokens.push(current);
                    current = String::new();
                }
                current.push(c);
            }
        }
        if !current.is_empty() { tokens.push(current); }
        tokens
    }

    fn compute_token_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        if tokens1.is_empty() || tokens2.is_empty() { return 0.0; }

        let mut total_score = 0.0;
        let mut matches = 0;

        for t1 in tokens1 {
            let mut best_score = 0.0;
            for t2 in tokens2 {
                if t1 == t2 {
                    best_score = 1.0;
                    break;
                }
                let distance = edit_distance(t1, t2);
                let max_len = max(t1.len(), t2.len());
                let score = if max_len > 0 { 1.0 - (distance as f64 / max_len as f64) } else { 0.0 };
                best_score = f64::max(best_score, score);
            }
            total_score += best_score;
            if best_score > 0.8 { matches += 1; }
        }

        let avg_score = total_score / tokens1.len() as f64;
        let match_ratio = matches as f64 / tokens1.len() as f64;
        avg_score * (1.0 + 0.5 * match_ratio)
    }
}

impl Resembler<String, String, ()> for Tokens {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let query_tokens = self.tokenize(&query.to_lowercase());
        let candidate_tokens = self.tokenize(&candidate.to_lowercase());
        let score = self.compute_token_similarity(&query_tokens, &candidate_tokens);

        let result = if score >= 1.0 {
            Resemblance::Perfect
        } else if score > 0.0 {
            Resemblance::Partial(score)
        } else {
            Resemblance::Disparity
        };

        Ok(result)
    }
}

#[derive(PartialEq)]
pub struct Initials {
    token_scorer: Tokens,
    max_initials_len: usize,
}

impl Default for Initials {
    fn default() -> Self {
        Self {
            token_scorer: Tokens::default(),
            max_initials_len: 5,
        }
    }
}

impl Resembler<String, String, ()> for Initials {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        if query.len() > self.max_initials_len {
            return Ok(Resemblance::Disparity);
        }

        let tokens = self.token_scorer.tokenize(&candidate.to_lowercase());
        if tokens.len() < query.len() {
            return Ok(Resemblance::Disparity);
        }

        let initials: String = tokens.iter().filter_map(|t| t.chars().next()).collect();
        if initials.contains(&query.to_lowercase()) {
            Ok(Resemblance::Partial(0.75))
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(PartialEq)]
pub struct Words {
    ignore_case: bool,
    min_word_len: usize,
    separators: Option<Vec<char>>,
    use_stemming: bool,
    stop_words: HashSet<String>,
}

impl Default for Words {
    fn default() -> Self {
        Self {
            ignore_case: true,
            min_word_len: 1,
            separators: None,
            use_stemming: false,
            stop_words: HashSet::new(),
        }
    }
}

impl Words {
    pub fn new(
        ignore_case: bool,
        min_word_len: usize,
        separators: Option<Vec<char>>,
        use_stemming: bool,
        stop_words: Option<Vec<&str>>,
    ) -> Self {
        Self {
            ignore_case,
            min_word_len,
            separators,
            use_stemming,
            stop_words: stop_words.map(|words| words.into_iter().map(String::from).collect()).unwrap_or_default(),
        }
    }

    pub fn with_case_sensitivity(ignore_case: bool) -> Self {
        Self { ignore_case, ..Default::default() }
    }

    fn extract_words(&self, text: &str) -> Vec<String> {
        let normalized = if self.ignore_case { text.to_lowercase() } else { text.to_string() };
        let mut words = Vec::new();
        let mut current = String::new();

        for c in normalized.chars() {
            let is_separator = c.is_whitespace() || self.separators.as_ref().map_or(false, |seps| seps.contains(&c));
            if is_separator {
                if !current.is_empty() {
                    self.process_word(&current, &mut words);
                    current.clear();
                }
            } else {
                current.push(c);
            }
        }
        if !current.is_empty() { self.process_word(&current, &mut words); }
        words
    }

    fn process_word(&self, word: &str, words: &mut Vec<String>) {
        if word.len() < self.min_word_len || self.stop_words.contains(word) { return; }
        let processed = if self.use_stemming { self.stem_word(word) } else { word.to_string() };
        words.push(processed);
    }

    fn stem_word(&self, word: &str) -> String {
        let mut result = word.to_string();
        for suffix in &["ing", "ed", "s", "es", "ies"] {
            if result.ends_with(suffix) && result.len() > suffix.len() + 2 {
                result.truncate(result.len() - suffix.len());
                break;
            }
        }
        result
    }

    fn weighted_jaccard(&self, query_words: &[String], candidate_words: &[String]) -> f64 {
        if query_words.is_empty() && candidate_words.is_empty() { return 1.0; }
        if query_words.is_empty() || candidate_words.is_empty() { return 0.0; }

        let mut common_weight = 0.0;
        for (i, q_word) in query_words.iter().enumerate() {
            for (j, c_word) in candidate_words.iter().enumerate() {
                if q_word == c_word {
                    let position_factor = 1.0 - (i as f64 - j as f64).abs() / max(query_words.len(), candidate_words.len()) as f64;
                    common_weight += 0.5 + 0.5 * position_factor;
                    break;
                }
            }
        }

        let union_size = query_words.len() + candidate_words.len() - common_weight as usize;
        common_weight / union_size as f64
    }
}

impl Resembler<String, String, ()> for Words {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let query_words = self.extract_words(query);
        let candidate_words = self.extract_words(candidate);

        if query_words.is_empty() && candidate_words.is_empty() {
            return Ok(Resemblance::Perfect);
        }
        if query_words.is_empty() || candidate_words.is_empty() {
            return Ok(Resemblance::Disparity);
        }

        let score = if query_words.len() <= 2 || candidate_words.len() <= 2 {
            let common_words = query_words.iter().filter(|w| candidate_words.contains(w)).count();
            let union_size = query_words.len() + candidate_words.len() - common_words;
            common_words as f64 / union_size as f64
        } else {
            self.weighted_jaccard(&query_words, &candidate_words)
        };

        let result = if score >= 1.0 {
            Resemblance::Perfect
        } else if score > 0.0 {
            Resemblance::Partial(score)
        } else {
            Resemblance::Disparity
        };

        Ok(result)
    }
}