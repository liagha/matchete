use {
    hashish::HashMap,
    crate::{
        assessor::{Resembler, Resemblance},
        prelude::string::utils::{edit_distance, keyboard::Layout},
    }
};
use core::cmp::{max, min};
use crate::prelude::string::Tokens;

/// Keyboard proximity matching
#[derive(Debug, PartialEq)]
pub struct Keyboard {
    layout: HashMap<char, Vec<char>>,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            layout: Layout::Qwerty.get_layout(),
        }
    }
}

impl Keyboard {
    pub fn new(layout_type: Layout) -> Self {
        Self {
            layout: layout_type.get_layout(),
        }
    }
}

impl Resembler<String, String, ()> for Keyboard {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let query_chars: Vec<char> = query.to_lowercase().chars().collect();
        let candidate_chars: Vec<char> = candidate.to_lowercase().chars().collect();

        if (query_chars.len() as isize - candidate_chars.len() as isize).abs() > 2 {
            return Ok(Resemblance::Disparity);
        }

        let distance = edit_distance(query, candidate);
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

/// Fuzzy search with multiple strategies
#[derive(Debug, PartialEq)]
pub struct Fuzzy {
    token_scorer: Tokens,
    min_score: f64,
}

impl Default for Fuzzy {
    fn default() -> Self {
        Self {
            token_scorer: Tokens::default(),
            min_score: 0.7,
        }
    }
}

impl Resembler<String, String, ()> for Fuzzy {
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
                let edit_score = 1.0 - (edit_distance(q_token, c_token) as f64 / max(q_token.len(), c_token.len()) as f64);
                best_score = f64::max(best_score, edit_score);
                if c_token.contains(q_token) {
                    let contain_score = q_token.len() as f64 / c_token.len() as f64 * 0.9;
                    best_score = best_score.max(contain_score);
                }
            }
            total_score += best_score;
            if best_score >= self.min_score { matched_tokens += 1; }
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
