/// Common similarity metrics used across Matchete example files.
/// These metrics demonstrate how to implement the `SimilarityMetric` trait
/// for strings and numbers.
use matchete::SimilarityMetric;
use std::collections::HashSet;

/// A simple Levenshtein-like string similarity metric (for demonstration).
/// Computes a score based on the difference in string lengths.
pub struct SimpleLevenshteinMetric;

impl SimilarityMetric<String, String> for SimpleLevenshteinMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let max_len = query.len().max(candidate.len()) as f64;
        if max_len == 0.0 {
            return 1.0;
        }
        // Simplified distance (not true Levenshtein, for demo purposes)
        let distance = (query.len() as i32 - candidate.len() as i32).abs() as f64;
        1.0 - (distance / max_len)
    }

    fn id(&self) -> &str {
        "simple_levenshtein"
    }
}

/// Jaccard similarity metric for strings based on character sets.
/// Computes the intersection over union of character sets.
pub struct JaccardMetric;

impl SimilarityMetric<String, String> for JaccardMetric {
    fn calculate(&self, query: &String, candidate: &String) -> f64 {
        let query_chars: HashSet<char> = query.chars().collect();
        let candidate_chars: HashSet<char> = candidate.chars().collect();
        let intersection = query_chars.intersection(&candidate_chars).count() as f64;
        let union = query_chars.union(&candidate_chars).count() as f64;
        if union == 0.0 {
            1.0
        } else {
            intersection / union
        }
    }

    fn id(&self) -> &str {
        "jaccard"
    }
}

/// Similarity metric for f64 numbers based on absolute difference.
/// Normalizes the difference within a fixed range (100.0).
pub struct NumericSimilarity;

impl SimilarityMetric<f64, f64> for NumericSimilarity {
    fn calculate(&self, query: &f64, candidate: &f64) -> f64 {
        let max_diff = 100.0; // Normalization range
        let diff = (query - candidate).abs();
        if diff > max_diff {
            0.0
        } else {
            1.0 - (diff / max_diff)
        }
    }

    fn id(&self) -> &str {
        "numeric"
    }
}