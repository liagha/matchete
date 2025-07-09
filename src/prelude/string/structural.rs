use crate::{
    assessor::{Resembler, Resemblance},
};

/// Prefix matching
#[derive(PartialEq)]
pub struct Prefix;

impl Resembler<String, String, ()> for Prefix {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
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

/// Suffix matching
#[derive(PartialEq)]
pub struct Suffix;

impl Resembler<String, String, ()> for Suffix {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
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

/// Substring matching
#[derive(PartialEq)]
pub struct Contains;

impl Resembler<String, String, ()> for Contains {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
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

/// N-gram overlap
#[derive(PartialEq)]
pub struct Sequential {
    size: usize,
}

impl Default for Sequential {
    fn default() -> Self {
        Self { size: 2 }
    }
}

impl Sequential {
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

impl Resembler<String, String, ()> for Sequential {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
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