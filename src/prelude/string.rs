use {
    crate::{
        assessor::{
            Resembler,
            Resemblance,
            Blend,
            Dimension,
        },
        prelude::{
            utils::{
                damerau_levenshtein_distance,
                KeyboardLayoutType,
            }
        }
    },
    core::{
        cmp::{max, min}
    },
    hashish::{
        HashMap,
        HashSet,
    },
};

#[derive(Debug, PartialEq)]
pub struct JaroWinkler {
    prefix_weight: f64,
}

impl Default for JaroWinkler {
    fn default() -> Self {
        Self { prefix_weight: 0.1 }
    }
}

impl JaroWinkler {
    pub fn new(prefix_weight: f64) -> Self {
        Self { prefix_weight }
    }

    fn compute_jaro(&self, str1: &str, str2: &str) -> f64 {
        let len1 = str1.chars().count();
        let len2 = str2.chars().count();

        if len1 == 0 && len2 == 0 { return 1.0; }
        if len1 == 0 || len2 == 0 { return 0.0; }

        let match_range = max(len1, len2) / 2 - 1;
        let chars1: Vec<char> = str1.chars().collect();
        let chars2: Vec<char> = str2.chars().collect();
        let mut matches1 = vec![false; len1];
        let mut matches2 = vec![false; len2];
        let mut match_count = 0;

        for i in 0..len1 {
            let start = i.saturating_sub(match_range).max(0);
            let end = min(i + match_range + 1, len2);

            for j in start..end {
                if !matches2[j] && chars1[i] == chars2[j] {
                    matches1[i] = true;
                    matches2[j] = true;
                    match_count += 1;
                    break;
                }
            }
        }

        if match_count == 0 { return 0.0; }

        let mut transpositions = 0;
        let mut k = 0;
        for i in 0..len1 {
            if matches1[i] {
                while !matches2[k] { k += 1; }
                if chars1[i] != chars2[k] { transpositions += 1; }
                k += 1;
            }
        }

        let m = match_count as f64;
        let t = transpositions as f64 / 2.0;
        (m / len1 as f64 + m / len2 as f64 + (m - t) / m) / 3.0
    }

    fn common_prefix_len(&self, str1: &str, str2: &str) -> usize {
        let max_prefix = 4;
        let chars1: Vec<char> = str1.chars().collect();
        let chars2: Vec<char> = str2.chars().collect();
        let min_len = min(chars1.len(), chars2.len()).min(max_prefix);

        (0..min_len).take_while(|&i| chars1[i] == chars2[i]).count()
    }

    fn compute_resemblance(&self, query: &str, candidate: &str) -> f64 {
        let jaro_score = self.compute_jaro(query, candidate);
        let prefix_len = self.common_prefix_len(query, candidate);
        jaro_score + prefix_len as f64 * self.prefix_weight * (1.0 - jaro_score)
    }
}

impl Resembler<String, String, ()> for JaroWinkler {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let score = self.compute_resemblance(query, candidate);
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

#[derive(Debug, PartialEq)]
pub struct Cosine {
    ngram_size: usize,
}

impl Default for Cosine {
    fn default() -> Self {
        Self { ngram_size: 2 }
    }
}

impl Cosine {
    pub fn new(ngram_size: usize) -> Self {
        Self { ngram_size: ngram_size.max(1) }
    }

    fn extract_ngrams(&self, text: &str) -> HashMap<String, usize> {
        let mut ngrams = HashMap::new();
        if text.len() < self.ngram_size {
            if !text.is_empty() { ngrams.insert(text.to_string(), 1); }
            return ngrams;
        }

        let chars: Vec<char> = text.chars().collect();
        for i in 0..=chars.len() - self.ngram_size {
            let ngram: String = chars[i..i + self.ngram_size].iter().collect();
            *ngrams.entry(ngram).or_insert(0) += 1;
        }
        ngrams
    }

    fn compute_resemblance(&self, query: &str, candidate: &str) -> f64 {
        let query_ngrams = self.extract_ngrams(query);
        let candidate_ngrams = self.extract_ngrams(candidate);

        if query_ngrams.is_empty() || candidate_ngrams.is_empty() {
            return if query.is_empty() && candidate.is_empty() { 1.0 } else { 0.0 };
        }

        let dot_product = query_ngrams.iter()
            .filter_map(|(ngram, count)| candidate_ngrams.get(ngram).map(|c| (*count as f64) * (*c as f64)))
            .sum::<f64>();

        let query_norm = query_ngrams.values().map(|c| (*c as f64).powi(2)).sum::<f64>().sqrt();
        let candidate_norm = candidate_ngrams.values().map(|c| (*c as f64).powi(2)).sum::<f64>().sqrt();

        if query_norm > 0.0 && candidate_norm > 0.0 {
            dot_product / (query_norm * candidate_norm)
        } else {
            0.0
        }
    }
}

impl Resembler<String, String, ()> for Cosine {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let score = self.compute_resemblance(query, candidate);
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

#[derive(Debug, PartialEq)]
pub struct ExactMatch;

impl Resembler<String, String, ()> for ExactMatch {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            Ok(Resemblance::Perfect)
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CaseInsensitive;

impl Resembler<String, String, ()> for CaseInsensitive {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query.to_lowercase() == candidate.to_lowercase() {
            Ok(Resemblance::Partial(0.95))
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Prefix;

impl Resembler<String, String, ()> for Prefix {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        if candidate.to_lowercase().starts_with(&query.to_lowercase()) {
            let score = 0.9 * f64::min(query.len() as f64 / candidate.len() as f64, 1.0);
            Ok(Resemblance::Partial(score))
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Suffix;

impl Resembler<String, String, ()> for Suffix {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        if candidate.to_lowercase().ends_with(&query.to_lowercase()) {
            let score = 0.85 * f64::min(query.len() as f64 / candidate.len() as f64, 1.0);
            Ok(Resemblance::Partial(score))
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Substring;

impl Resembler<String, String, ()> for Substring {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        if candidate.to_lowercase().contains(&query.to_lowercase()) {
            let score = 0.8 * f64::min(query.len() as f64 / candidate.len() as f64, 1.0);
            Ok(Resemblance::Partial(score))
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EditDistance;

impl Resembler<String, String, ()> for EditDistance {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let distance = damerau_levenshtein_distance(query, candidate);
        let max_len = max(query.len(), candidate.len());
        let score = if max_len == 0 { 1.0 } else { 1.0 - (distance as f64 / max_len as f64) };

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

#[derive(Debug, PartialEq)]
pub struct TokenSimilarity {
    separators: Vec<char>,
}

impl Default for TokenSimilarity {
    fn default() -> Self {
        Self { separators: vec!['_', '-', '.', ' '] }
    }
}

impl TokenSimilarity {
    pub fn new(separators: Vec<char>) -> Self {
        Self { separators }
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
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
                let distance = damerau_levenshtein_distance(t1, t2);
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

impl Resembler<String, String, ()> for TokenSimilarity {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
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

#[derive(Debug, PartialEq)]
pub struct Acronym {
    token_scorer: TokenSimilarity,
    max_acronym_len: usize,
}

impl Default for Acronym {
    fn default() -> Self {
        Self {
            token_scorer: TokenSimilarity::default(),
            max_acronym_len: 5,
        }
    }
}

impl Resembler<String, String, ()> for Acronym {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        if query.len() > self.max_acronym_len {
            return Ok(Resemblance::Disparity);
        }

        let tokens = self.token_scorer.tokenize(&candidate.to_lowercase());
        if tokens.len() < query.len() {
            return Ok(Resemblance::Disparity);
        }

        let acronym: String = tokens.iter().filter_map(|t| t.chars().next()).collect();
        if acronym.contains(&query.to_lowercase()) {
            Ok(Resemblance::Partial(0.75))
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct KeyboardProximity {
    layout: HashMap<char, Vec<char>>,
    layout_type: KeyboardLayoutType,
}

impl Default for KeyboardProximity {
    fn default() -> Self {
        Self {
            layout: KeyboardLayoutType::Qwerty.get_layout(),
            layout_type: KeyboardLayoutType::Qwerty,
        }
    }
}

impl KeyboardProximity {
    pub fn new(layout_type: KeyboardLayoutType) -> Self {
        Self {
            layout: layout_type.get_layout(),
            layout_type,
        }
    }
}

impl Resembler<String, String, ()> for KeyboardProximity {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let query_chars: Vec<char> = query.to_lowercase().chars().collect();
        let candidate_chars: Vec<char> = candidate.to_lowercase().chars().collect();

        if (query_chars.len() as isize - candidate_chars.len() as isize).abs() > 2 {
            return Ok(Resemblance::Disparity);
        }

        let distance = damerau_levenshtein_distance(query, candidate);
        if distance > 3 {
            return Ok(Resemblance::Disparity);
        }

        let mut adjacent_count = 0;
        let max_comparisons = min(query_chars.len(), candidate_chars.len());
        for i in 0..max_comparisons {
            if query_chars[i] == candidate_chars[i] { continue; }
            if let Some(neighbors) = self.layout.get(&query_chars[i]) {
                if neighbors.contains(&candidate_chars[i]) { adjacent_count += 1; }
            }
        }

        let differing_chars = distance;
        if differing_chars == 0 { return Ok(Resemblance::Perfect); }

        let keyboard_factor = adjacent_count as f64 / differing_chars as f64;
        let length_similarity = 1.0 - ((query_chars.len() as isize - candidate_chars.len() as isize).abs() as f64 / max(query_chars.len(), candidate_chars.len()) as f64);
        let base_score = 1.0 - (distance as f64 / max(query_chars.len(), candidate_chars.len()) as f64);
        let score = base_score * (1.0 + 0.3 * keyboard_factor) * length_similarity;

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

#[derive(Debug, PartialEq)]
pub struct FuzzySearch {
    token_scorer: TokenSimilarity,
    min_token_score: f64,
}

impl Default for FuzzySearch {
    fn default() -> Self {
        Self {
            token_scorer: TokenSimilarity::default(),
            min_token_score: 0.7,
        }
    }
}

impl Resembler<String, String, ()> for FuzzySearch {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let query_tokens = self.token_scorer.tokenize(&query.to_lowercase());
        let candidate_tokens = self.token_scorer.tokenize(&candidate.to_lowercase());

        if query_tokens.is_empty() || candidate_tokens.is_empty() {
            return Ok(Resemblance::Disparity);
        }

        let mut matched_tokens = 0;
        let mut total_score = 0.0;

        for q_token in &query_tokens {
            let mut best_score = 0.0;
            for c_token in &candidate_tokens {
                let edit_score = 1.0 - (damerau_levenshtein_distance(q_token, c_token) as f64 / max(q_token.len(), c_token.len()) as f64);
                best_score = f64::max(best_score, edit_score);
                if c_token.contains(q_token) {
                    let contain_score = q_token.len() as f64 / c_token.len() as f64 * 0.9;
                    best_score = best_score.max(contain_score);
                }
            }
            total_score += best_score;
            if best_score >= self.min_token_score { matched_tokens += 1; }
        }

        let coverage = matched_tokens as f64 / query_tokens.len() as f64;
        let avg_score = total_score / query_tokens.len() as f64;
        let score = coverage * avg_score * (0.7 + 0.3 * coverage);

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

#[derive(Debug, PartialEq)]
pub struct Phonetic {
    mode: PhoneticMode,
}

#[derive(Debug, PartialEq)]
pub enum PhoneticMode {
    Soundex,
    DoubleMetaphone,
}

impl Default for Phonetic {
    fn default() -> Self {
        Self { mode: PhoneticMode::Soundex }
    }
}

impl Phonetic {
    pub fn new(mode: PhoneticMode) -> Self {
        Self { mode }
    }

    fn compute_soundex(&self, text: &str) -> String {
        if text.is_empty() { return "0000".to_string(); }

        let mut result = String::new();
        let mut prev_code = 0;
        for (i, c) in text.to_lowercase().chars().enumerate() {
            let code = match c {
                'b' | 'f' | 'p' | 'v' => 1,
                'c' | 'g' | 'j' | 'k' | 'q' | 's' | 'x' | 'z' => 2,
                'd' | 't' => 3,
                'l' => 4,
                'm' | 'n' => 5,
                'r' => 6,
                _ => 0,
            };

            if i == 0 {
                result.push(c.to_ascii_uppercase());
            } else if code != 0 && code != prev_code {
                result.push(char::from_digit(code, 10).unwrap());
            }

            prev_code = code;
            if result.len() >= 4 { break; }
        }

        while result.len() < 4 {
            result.push('0');
        }
        result
    }
}

impl Resembler<String, String, ()> for Phonetic {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let result = match self.mode {
            PhoneticMode::Soundex => {
                let query_code = self.compute_soundex(query);
                let candidate_code = self.compute_soundex(candidate);
                if query_code == candidate_code {
                    Resemblance::Partial(0.85)
                } else {
                    let common_prefix_len = query_code.chars().zip(candidate_code.chars())
                        .take_while(|(c1, c2)| c1 == c2)
                        .count();
                    if common_prefix_len > 0 {
                        Resemblance::Partial(0.6 * (common_prefix_len as f64 / 4.0))
                    } else {
                        Resemblance::Disparity
                    }
                }
            }
            PhoneticMode::DoubleMetaphone => {
                if query.to_lowercase() == candidate.to_lowercase() {
                    return Ok(Resemblance::Perfect);
                }
                let query_code = self.compute_soundex(query);
                let candidate_code = self.compute_soundex(candidate);
                if query_code == candidate_code {
                    Resemblance::Partial(0.8)
                } else {
                    Resemblance::Disparity
                }
            }
        };

        Ok(result)
    }
}

#[derive(Debug, PartialEq)]
pub struct NGram {
    size: usize,
}

impl Default for NGram {
    fn default() -> Self {
        Self { size: 2 }
    }
}

impl NGram {
    pub fn new(size: usize) -> Self {
        Self { size }
    }

    fn generate_ngrams(&self, text: &str) -> Vec<String> {
        if text.len() < self.size { return vec![text.to_string()]; }

        let chars: Vec<char> = text.chars().collect();
        (0..=chars.len() - self.size)
            .map(|i| chars[i..i + self.size].iter().collect())
            .collect()
    }
}

impl Resembler<String, String, ()> for NGram {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        if query.is_empty() && candidate.is_empty() {
            return Ok(Resemblance::Perfect);
        }
        if query.is_empty() || candidate.is_empty() {
            return Ok(Resemblance::Disparity);
        }

        let query_ngrams = self.generate_ngrams(&query.to_lowercase());
        let candidate_ngrams = self.generate_ngrams(&candidate.to_lowercase());

        if query_ngrams.is_empty() || candidate_ngrams.is_empty() {
            return Ok(Resemblance::Disparity);
        }

        let intersection = query_ngrams.iter().filter(|ngram| candidate_ngrams.contains(ngram)).count();
        let score = 2.0 * intersection as f64 / (query_ngrams.len() + candidate_ngrams.len()) as f64;

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

#[derive(Debug, PartialEq)]
pub struct WordOverlap {
    ignore_case: bool,
    min_word_len: usize,
    separators: Option<Vec<char>>,
    use_stemming: bool,
    stop_words: HashSet<String>,
}

impl Default for WordOverlap {
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

impl WordOverlap {
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

impl Resembler<String, String, ()> for WordOverlap {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
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

#[derive(Debug)]
pub struct FullMatcher {
    blend: Blend<String, String, ()>,
}

impl Default for FullMatcher {
    fn default() -> Self {
        let dimensions = vec![
            Dimension::new(JaroWinkler::default(), 0.2),
            Dimension::new(Cosine::default(), 0.15),
            Dimension::new(ExactMatch, 0.1),
            Dimension::new(CaseInsensitive, 0.1),
            Dimension::new(Prefix, 0.1),
            Dimension::new(Suffix, 0.05),
            Dimension::new(Substring, 0.05),
            Dimension::new(EditDistance, 0.1),
            Dimension::new(TokenSimilarity::default(), 0.1),
            Dimension::new(Acronym::default(), 0.05),
            Dimension::new(KeyboardProximity::default(), 0.05),
            Dimension::new(FuzzySearch::default(), 0.1),
            Dimension::new(Phonetic::default(), 0.05),
            Dimension::new(NGram::default(), 0.05),
            Dimension::new(WordOverlap::default(), 0.1),
        ];
        Self {
            blend: Blend::weighted(dimensions),
        }
    }
}

impl Resembler<String, String, ()> for FullMatcher {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        self.blend.resemblance(query, candidate)
    }
}