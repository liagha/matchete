use {
    core::cmp::{max, min},
    hashish::{
        HashMap,
        HashSet
    },
    crate::{
        Scorer,
        utils::{
            damerau_levenshtein_distance, KeyboardLayoutType
        }
    }
};

/// Jaro-Winkler similarity scorer for strings
#[derive(Debug)]
pub struct JaroWinklerScorer {
    prefix_scale: f64,
}

impl Default for JaroWinklerScorer {
    fn default() -> Self {
        Self { prefix_scale: 0.1 } // Standard prefix scaling factor
    }
}

impl JaroWinklerScorer {
    pub fn new(prefix_scale: f64) -> Self {
        Self { prefix_scale }
    }

    fn jaro_distance(&self, s1: &str, s2: &str) -> f64 {
        let s1_len = s1.chars().count();
        let s2_len = s2.chars().count();

        if s1_len == 0 && s2_len == 0 {
            return 1.0;
        }

        if s1_len == 0 || s2_len == 0 {
            return 0.0;
        }

        // Maximum distance to consider characters as matching
        let match_distance = (s1_len.max(s2_len) / 2).max(1) - 1;

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        // Track matches
        let mut s1_matches = vec![false; s1_len];
        let mut s2_matches = vec![false; s2_len];

        // Count matching characters
        let mut matches = 0;
        for i in 0..s1_len {
            let start = i.saturating_sub(match_distance).max(0);
            let end = (i + match_distance + 1).min(s2_len);

            for j in start..end {
                if !s2_matches[j] && s1_chars[i] == s2_chars[j] {
                    s1_matches[i] = true;
                    s2_matches[j] = true;
                    matches += 1;
                    break;
                }
            }
        }

        if matches == 0 {
            return 0.0;
        }

        // Count transpositions
        let mut transpositions = 0;
        let mut k = 0;

        for i in 0..s1_len {
            if s1_matches[i] {
                while !s2_matches[k] {
                    k += 1;
                }

                if s1_chars[i] != s2_chars[k] {
                    transpositions += 1;
                }

                k += 1;
            }
        }

        // Calculate Jaro similarity
        let m = matches as f64;
        let t = transpositions as f64 / 2.0;

        if matches == 0 {
            0.0
        } else {
            (m / s1_len as f64 + m / s2_len as f64 + (m - t) / m) / 3.0
        }
    }

    fn get_common_prefix_length(&self, s1: &str, s2: &str) -> usize {
        let max_prefix_len = 4; // Standard prefix length for Jaro-Winkler

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        let min_len = s1_chars.len().min(s2_chars.len()).min(max_prefix_len);

        let mut prefix_len = 0;
        for i in 0..min_len {
            if s1_chars[i] == s2_chars[i] {
                prefix_len += 1;
            } else {
                break;
            }
        }

        prefix_len
    }
}

impl Scorer<String, String> for JaroWinklerScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let jaro_dist = self.jaro_distance(query, candidate);

        // Apply Winkler modification (rewards strings with common prefixes)
        let prefix_len = self.get_common_prefix_length(query, candidate);

        jaro_dist + (prefix_len as f64 * self.prefix_scale * (1.0 - jaro_dist))
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

impl Scorer<&str, String> for JaroWinklerScorer {
    fn score(&self, query: &&str, candidate: &String) -> f64 {
        let jaro_dist = self.jaro_distance(query, candidate);
        let prefix_len = self.get_common_prefix_length(query, candidate);
        jaro_dist + (prefix_len as f64 * self.prefix_scale * (1.0 - jaro_dist))
    }

    fn exact(&self, query: &&str, candidate: &String) -> bool {
        *query == candidate
    }
}

/// Cosine similarity scorer for strings using character n-grams
#[derive(Debug)]
pub struct CosineScorer {
    n_gram_size: usize,
}

impl Default for CosineScorer {
    fn default() -> Self {
        Self { n_gram_size: 2 } // Default to bigrams
    }
}

impl CosineScorer {
    pub fn new(n_gram_size: usize) -> Self {
        Self { n_gram_size: n_gram_size.max(1) }
    }

    fn get_n_grams(&self, text: &str) -> HashMap<String, usize> {
        let mut n_grams = HashMap::new();

        if text.len() < self.n_gram_size {
            if !text.is_empty() {
                *n_grams.entry(text.to_string()).or_insert(0) += 1;
            }
            return n_grams;
        }

        let chars: Vec<char> = text.chars().collect();

        for i in 0..=(chars.len() - self.n_gram_size) {
            let n_gram: String = chars[i..(i + self.n_gram_size)].iter().collect();
            *n_grams.entry(n_gram).or_insert(0) += 1;
        }

        n_grams
    }
}

impl Scorer<String, String> for CosineScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let query_n_grams = self.get_n_grams(query);
        let candidate_n_grams = self.get_n_grams(candidate);

        if query_n_grams.is_empty() || candidate_n_grams.is_empty() {
            return if query.is_empty() && candidate.is_empty() { 1.0 } else { 0.0 };
        }

        // Calculate dot product
        let mut dot_product = 0.0;
        for (n_gram, query_count) in &query_n_grams {
            if let Some(candidate_count) = candidate_n_grams.get(n_gram) {
                dot_product += (*query_count as f64) * (*candidate_count as f64);
            }
        }

        // Calculate magnitudes
        let query_magnitude: f64 = query_n_grams.values().map(|count| (*count as f64).powi(2)).sum::<f64>().sqrt();
        let candidate_magnitude: f64 = candidate_n_grams.values().map(|count| (*count as f64).powi(2)).sum::<f64>().sqrt();

        // Calculate cosine similarity
        if query_magnitude > 0.0 && candidate_magnitude > 0.0 {
            dot_product / (query_magnitude * candidate_magnitude)
        } else {
            0.0
        }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

#[derive(Debug)]
pub struct ExactMatchScorer;

impl Scorer<String, String> for ExactMatchScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        if query == candidate { 1.0 } else { 0.0 }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

impl Scorer<&str, String> for ExactMatchScorer {
    fn score(&self, query: &&str, candidate: &String) -> f64 {
        if *query == candidate { 1.0 } else { 0.0 }
    }

    fn exact(&self, query: &&str, candidate: &String) -> bool {
        *query == candidate
    }
}

#[derive(Debug)]
pub struct CaseInsensitiveScorer;

impl Scorer<String, String> for CaseInsensitiveScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query.to_lowercase() == candidate.to_lowercase()
    }
}

impl Scorer<&str, String> for CaseInsensitiveScorer {
    fn score(&self, query: &&str, candidate: &String) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn exact(&self, query: &&str, candidate: &String) -> bool {
        query.to_lowercase() == candidate.to_lowercase()
    }
}

impl Scorer<String, &str> for CaseInsensitiveScorer {
    fn score(&self, query: &String, candidate: &&str) -> f64 {
        if query.to_lowercase() == candidate.to_lowercase() { 0.95 } else { 0.0 }
    }

    fn exact(&self, query: &String, candidate: &&str) -> bool {
        query.to_lowercase() == candidate.to_lowercase()
    }
}

#[derive(Debug)]
pub struct PrefixScorer;

impl Scorer<String, String> for PrefixScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.starts_with(&query_lower) {
            0.9 * (query.len() as f64 / candidate.len() as f64).min(1.0)
        } else {
            0.0
        }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

impl Scorer<&str, String> for PrefixScorer {
    fn score(&self, query: &&str, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.starts_with(&query_lower) {
            0.9 * (query.len() as f64 / candidate.len() as f64).min(1.0)
        } else {
            0.0
        }
    }

    fn exact(&self, query: &&str, candidate: &String) -> bool {
        *query == candidate
    }
}

#[derive(Debug)]
pub struct SuffixScorer;

impl Scorer<String, String> for SuffixScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.ends_with(&query_lower) {
            0.85 * (query.len() as f64 / candidate.len() as f64).min(1.0)
        } else {
            0.0
        }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

#[derive(Debug)]
pub struct SubstringScorer;

impl Scorer<String, String> for SubstringScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower.contains(&query_lower) {
            0.8 * (query.len() as f64 / candidate.len() as f64).min(1.0)
        } else {
            0.0
        }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

#[derive(Debug)]
pub struct EditDistanceScorer;

impl Scorer<String, String> for EditDistanceScorer {
    fn score(&self, s1: &String, s2: &String) -> f64 {
        let distance = damerau_levenshtein_distance(s1, s2);
        let max_len = max(s1.len(), s2.len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }

    fn exact(&self, s1: &String, s2: &String) -> bool {
        s1 == s2
    }
}

#[derive(Debug)]
pub struct TokenSimilarityScorer {
    pub separators: Vec<char>,
}

impl Default for TokenSimilarityScorer {
    fn default() -> Self {
        TokenSimilarityScorer {
            separators: vec!['_', '-', '.', ' '],
        }
    }
}

impl TokenSimilarityScorer {
    pub fn new(separators: Vec<char>) -> Self {
        TokenSimilarityScorer { separators }
    }

    pub fn split_on_separators(&self, s: &str) -> Vec<String> {
        let mut tokens: Vec<String> = Vec::new();
        let mut current = String::new();

        for c in s.chars() {
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

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    pub fn token_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }

        let mut total_sim = 0.0;
        let mut matches = 0;

        for t1 in tokens1 {
            let mut best_sim : f64 = 0.0;

            for t2 in tokens2 {
                if t1 == t2 {
                    best_sim = 1.0;
                    break;
                }

                let edit_distance = damerau_levenshtein_distance(t1, t2);
                let max_len = max(t1.len(), t2.len());
                let token_sim = if max_len > 0 {
                    1.0 - (edit_distance as f64 / max_len as f64)
                } else {
                    0.0
                };

                best_sim = best_sim.max(token_sim);
            }

            total_sim += best_sim;
            if best_sim > 0.8 {
                matches += 1;
            }
        }

        let token_sim = if !tokens1.is_empty() {
            total_sim / tokens1.len() as f64
        } else {
            0.0
        };

        let match_ratio = if !tokens1.is_empty() {
            matches as f64 / tokens1.len() as f64
        } else {
            0.0
        };

        token_sim * (1.0 + 0.5 * match_ratio)
    }
}

impl Scorer<String, String> for TokenSimilarityScorer {
    fn score(&self, s1: &String, s2: &String) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        let s1_tokens = self.split_on_separators(&s1_lower);
        let s2_tokens = self.split_on_separators(&s2_lower);

        self.token_similarity(&s1_tokens, &s2_tokens)
    }

    fn exact(&self, s1: &String, s2: &String) -> bool {
        s1 == s2
    }
}

#[derive(Debug)]
pub struct AcronymScorer {
    pub token_scorer: TokenSimilarityScorer,
    pub max_acronym_length: usize,
}

impl Default for AcronymScorer {
    fn default() -> Self {
        AcronymScorer {
            token_scorer: TokenSimilarityScorer::default(),
            max_acronym_length: 5,
        }
    }
}

impl Scorer<String, String> for AcronymScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        if query.len() > self.max_acronym_length {
            return 0.0;
        }

        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        let tokens = self.token_scorer.split_on_separators(&candidate_lower);

        if tokens.len() < query_lower.len() {
            return 0.0;
        }

        let first_letters: String = tokens.iter()
            .filter_map(|token| token.chars().next())
            .collect();

        if first_letters.contains(&query_lower) {
            return 0.75;
        }

        0.0
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

#[derive(Debug)]
pub struct KeyboardProximityScorer {
    pub keyboard_layout: HashMap<char, Vec<char>>,
    pub layout_type: KeyboardLayoutType,
}

impl Default for KeyboardProximityScorer {
    fn default() -> Self {
        KeyboardProximityScorer {
            keyboard_layout: KeyboardLayoutType::Qwerty.get_layout(),
            layout_type: KeyboardLayoutType::Qwerty,
        }
    }
}

impl KeyboardProximityScorer {
    pub fn new(layout_type: KeyboardLayoutType) -> Self {
        KeyboardProximityScorer {
            keyboard_layout: layout_type.get_layout(),
            layout_type,
        }
    }
}

impl Scorer<String, String> for KeyboardProximityScorer {
    fn score(&self, s1: &String, s2: &String) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        if (s1_lower.len() as isize - s2_lower.len() as isize).abs() > 2 {
            return 0.0;
        }

        let s1_chars: Vec<char> = s1_lower.chars().collect();
        let s2_chars: Vec<char> = s2_lower.chars().collect();

        let edit_distance = damerau_levenshtein_distance(&s1_lower, &s2_lower);

        if edit_distance > 3 {
            return 0.0;
        }

        let mut adjacency_count = 0;
        let max_comparisons = min(s1_chars.len(), s2_chars.len());

        for i in 0..max_comparisons {
            if s1_chars[i] == s2_chars[i] {
                continue;
            }

            if let Some(neighbors) = self.keyboard_layout.get(&s1_chars[i]) {
                if neighbors.contains(&s2_chars[i]) {
                    adjacency_count += 1;
                }
            }
        }

        let differing_chars = edit_distance;

        if differing_chars == 0 {
            1.0
        } else {
            let keyboard_factor = adjacency_count as f64 / differing_chars as f64;
            let length_similarity = 1.0 - ((s1_chars.len() as isize - s2_chars.len() as isize).abs() as f64 / max(s1_chars.len(), s2_chars.len()) as f64);

            let base_similarity = 1.0 - (edit_distance as f64 / max(s1_chars.len(), s2_chars.len()) as f64);
            base_similarity * (1.0 + 0.3 * keyboard_factor) * length_similarity
        }
    }

    fn exact(&self, s1: &String, s2: &String) -> bool {
        s1 == s2
    }
}

#[derive(Debug)]
pub struct FuzzySearchScorer {
    pub token_scorer: TokenSimilarityScorer,
    pub min_token_similarity: f64,
}

impl Default for FuzzySearchScorer {
    fn default() -> Self {
        FuzzySearchScorer {
            token_scorer: TokenSimilarityScorer::default(),
            min_token_similarity: 0.7,
        }
    }
}

impl Scorer<String, String> for FuzzySearchScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();

        let query_tokens = self.token_scorer.split_on_separators(&query_lower);
        let candidate_tokens = self.token_scorer.split_on_separators(&candidate_lower);

        if query_tokens.is_empty() || candidate_tokens.is_empty() {
            return 0.0;
        }

        let mut matched_tokens = 0;
        let mut total_similarity = 0.0;

        for q_token in &query_tokens {
            let mut best_match = 0.0;

            for c_token in &candidate_tokens {
                let edit_sim = 1.0 - (damerau_levenshtein_distance(q_token, c_token) as f64
                    / max(q_token.len(), c_token.len()) as f64);

                if edit_sim > best_match {
                    best_match = edit_sim;
                }

                if c_token.contains(q_token) {
                    let contain_score = q_token.len() as f64 / c_token.len() as f64 * 0.9;
                    best_match = best_match.max(contain_score);
                }
            }

            total_similarity += best_match;
            if best_match >= self.min_token_similarity {
                matched_tokens += 1;
            }
        }

        let coverage = matched_tokens as f64 / query_tokens.len() as f64;
        let avg_similarity = total_similarity / query_tokens.len() as f64;

        coverage * avg_similarity * (0.7 + 0.3 * coverage)
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

#[derive(Debug)]
pub struct PhoneticScorer {
    pub mode: PhoneticMode,
}

#[derive(Debug)]
pub enum PhoneticMode {
    Soundex,
    DoubleMetaphone,
}

impl Default for PhoneticScorer {
    fn default() -> Self {
        PhoneticScorer {
            mode: PhoneticMode::Soundex,
        }
    }
}

impl PhoneticScorer {
    pub fn new(mode: PhoneticMode) -> Self {
        PhoneticScorer { mode }
    }

    fn soundex(&self, s: &str) -> String {
        if s.is_empty() {
            return "0000".to_string();
        }

        let mut result = String::new();
        let mut prev_code = 0;

        for (i, c) in s.to_lowercase().chars().enumerate() {
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

            if result.len() >= 4 {
                break;
            }
        }

        while result.len() < 4 {
            result.push('0');
        }

        result
    }
}

impl Scorer<String, String> for PhoneticScorer {
    fn score(&self, s1: &String, s2: &String) -> f64 {
        match self.mode {
            PhoneticMode::Soundex => {
                let s1_code = self.soundex(s1);
                let s2_code = self.soundex(s2);

                if s1_code == s2_code {
                    0.85
                } else {
                    let common_prefix_len = s1_code.chars().zip(s2_code.chars())
                        .take_while(|(c1, c2)| c1 == c2)
                        .count();

                    if common_prefix_len > 0 {
                        0.6 * (common_prefix_len as f64 / 4.0)
                    } else {
                        0.0
                    }
                }
            },
            PhoneticMode::DoubleMetaphone => {
                // Simplified double metaphone implementation
                if s1.to_lowercase() == s2.to_lowercase() {
                    return 1.0;
                }

                // Just use soundex as fallback
                let s1_code = self.soundex(s1);
                let s2_code = self.soundex(s2);

                if s1_code == s2_code {
                    0.8
                } else {
                    0.0
                }
            }
        }
    }

    fn exact(&self, s1: &String, s2: &String) -> bool {
        s1 == s2
    }
}

#[derive(Debug)]
pub struct NGramScorer {
    pub n: usize,
}

impl Default for NGramScorer {
    fn default() -> Self {
        NGramScorer { n: 2 }
    }
}

impl NGramScorer {
    pub fn new(n: usize) -> Self {
        NGramScorer { n }
    }

    fn generate_ngrams(&self, s: &str) -> Vec<String> {
        if s.len() < self.n {
            return vec![s.to_string()];
        }

        let chars: Vec<char> = s.chars().collect();
        let mut ngrams = Vec::new();

        for i in 0..=chars.len() - self.n {
            let ngram: String = chars[i..i + self.n].iter().collect();
            ngrams.push(ngram);
        }

        ngrams
    }
}

impl Scorer<String, String> for NGramScorer {
    fn score(&self, s1: &String, s2: &String) -> f64 {
        if s1.is_empty() || s2.is_empty() {
            return if s1.is_empty() && s2.is_empty() { 1.0 } else { 0.0 };
        }

        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        let s1_ngrams = self.generate_ngrams(&s1_lower);
        let s2_ngrams = self.generate_ngrams(&s2_lower);

        if s1_ngrams.is_empty() || s2_ngrams.is_empty() {
            return 0.0;
        }

        let mut intersection = 0;

        for ngram in &s1_ngrams {
            if s2_ngrams.contains(ngram) {
                intersection += 1;
            }
        }

        (2.0 * intersection as f64) / (s1_ngrams.len() + s2_ngrams.len()) as f64
    }

    fn exact(&self, s1: &String, s2: &String) -> bool {
        s1 == s2
    }
}

/// Word overlap similarity scorer using Jaccard similarity with customizable tokenization
#[derive(Debug)]
pub struct WordOverlapScorer {
    /// Whether to ignore case when comparing words
    ignore_case: bool,
    /// Minimum length of words to consider
    min_word_length: usize,
    /// Custom tokenization characters (in addition to whitespace)
    custom_separators: Option<Vec<char>>,
    /// Whether to use stemming for word comparison
    use_stemming: bool,
    /// Stopwords to ignore in comparison
    stopwords: HashSet<String>,
}

impl Default for WordOverlapScorer {
    fn default() -> Self {
        Self {
            ignore_case: true,
            min_word_length: 1,
            custom_separators: None,
            use_stemming: false,
            stopwords: HashSet::new(),
        }
    }
}

// Remove the duplicate Default implementation and fix the WordOverlapScorer
impl WordOverlapScorer {
    /// Create a new WordOverlapScorer with custom settings
    pub fn new(
        ignore_case: bool,
        min_word_length: usize,
        custom_separators: Option<Vec<char>>,
        use_stemming: bool,
        stopwords: Option<Vec<&str>>,
    ) -> Self {
        Self {
            ignore_case,
            min_word_length,
            custom_separators,
            use_stemming,
            stopwords: stopwords
                .map(|words| words.into_iter().map(String::from).collect())
                .unwrap_or_default(),
        }
    }

    /// Create a simple WordOverlapScorer with just case sensitivity setting
    pub fn with_case_sensitivity(ignore_case: bool) -> Self {
        Self {
            ignore_case,
            ..Default::default()
        }
    }

    /// Tokenize text into words based on configuration
    fn get_words(&self, text: &str) -> Vec<String> {
        let normalized = if self.ignore_case {
            text.to_lowercase()
        } else {
            text.to_string()
        };

        let mut result = Vec::new();
        let mut current_word = String::new();

        for c in normalized.chars() {
            let is_separator = c.is_whitespace() ||
                self.custom_separators.as_ref()
                    .map_or(false, |seps| seps.contains(&c));

            if is_separator {
                if !current_word.is_empty() {
                    self.add_processed_word(&current_word, &mut result);
                    current_word.clear();
                }
            } else {
                current_word.push(c);
            }
        }

        if !current_word.is_empty() {
            self.add_processed_word(&current_word, &mut result);
        }

        result
    }

    /// Process and add a word to the result if it meets criteria
    fn add_processed_word(&self, word: &str, result: &mut Vec<String>) {
        if word.len() < self.min_word_length {
            return;
        }

        if self.stopwords.contains(word) {
            return;
        }

        let processed = if self.use_stemming {
            self.apply_stemming(word)
        } else {
            word.to_string()
        };

        result.push(processed);
    }

    /// Apply basic stemming (very simplified Porter stemming)
    fn apply_stemming(&self, word: &str) -> String {
        let mut result = word.to_string();

        for suffix in &["ing", "ed", "s", "es", "ies"] {
            if result.ends_with(suffix) && result.len() > suffix.len() + 2 {
                result.truncate(result.len() - suffix.len());
                break;
            }
        }

        result
    }

    /// Calculate weighted Jaccard similarity with position awareness
    fn weighted_jaccard(&self, query_words: &[String], candidate_words: &[String]) -> f64 {
        if query_words.is_empty() && candidate_words.is_empty() {
            return 1.0;
        }

        if query_words.is_empty() || candidate_words.is_empty() {
            return 0.0;
        }

        let mut common_weight = 0.0;

        for (i, q_word) in query_words.iter().enumerate() {
            for (j, c_word) in candidate_words.iter().enumerate() {
                if q_word == c_word {
                    let position_factor = 1.0 - (i as f64 - j as f64).abs() /
                        (query_words.len().max(candidate_words.len()) as f64);

                    common_weight += 1.0 * (0.5 + 0.5 * position_factor);
                    break;
                }
            }
        }

        let union_size = query_words.len() + candidate_words.len() - common_weight as usize;
        common_weight / union_size as f64
    }
}

impl Scorer<String, String> for WordOverlapScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let query_words = self.get_words(query);
        let candidate_words = self.get_words(candidate);

        if query_words.is_empty() && candidate_words.is_empty() {
            return 1.0;
        }

        if query_words.is_empty() || candidate_words.is_empty() {
            return 0.0;
        }

        // Use standard Jaccard similarity for simple cases
        if query_words.len() <= 2 || candidate_words.len() <= 2 {
            let mut common_words = 0;
            for q_word in &query_words {
                if candidate_words.contains(q_word) {
                    common_words += 1;
                }
            }

            let union_size = query_words.len() + candidate_words.len() - common_words;
            common_words as f64 / union_size as f64
        } else {
            self.weighted_jaccard(&query_words, &candidate_words)
        }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

impl Scorer<&str, String> for WordOverlapScorer {
    fn score(&self, query: &&str, candidate: &String) -> f64 {
        let query_str = query.to_string();
        self.score(&query_str, candidate)
    }

    fn exact(&self, query: &&str, candidate: &String) -> bool {
        *query == candidate
    }
}

impl Scorer<String, &str> for WordOverlapScorer {
    fn score(&self, query: &String, candidate: &&str) -> f64 {
        let candidate_str = candidate.to_string();
        self.score(query, &candidate_str)
    }

    fn exact(&self, query: &String, candidate: &&str) -> bool {
        query == *candidate
    }
}