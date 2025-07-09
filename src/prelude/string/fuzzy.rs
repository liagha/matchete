use {
    hashish::HashMap,

    crate::{
        assessor::{Resembler, Resemblance},
        prelude::string::utils::edit_distance,
    }
};
use core::cmp::{max, min};

/// Jaro-Winkler similarity
#[derive(Debug, PartialEq)]
pub struct Jaro {
    prefix_weight: f64,
}

impl Default for Jaro {
    fn default() -> Self {
        Self { prefix_weight: 0.1 }
    }
}

impl Jaro {
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

impl Resembler<String, String, ()> for Jaro {
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

/// Cosine similarity using n-grams
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

/// Edit distance matcher
#[derive(Debug, PartialEq)]
pub struct Levenshtein;

impl Resembler<String, String, ()> for Levenshtein {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let distance = edit_distance(query, candidate);
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