use crate::{
    Matcher, MatcherBuilder, SimilarityMetric,
    LevenshteinSimilarity, JaroWinklerSimilarity,
    CosineSimilarity, SoundexSimilarity, WordOverlapSimilarity,
    CompositeSimilarity, CompositeStrategy, ExactMatchMetric
};

pub struct MatcherPresets;

impl MatcherPresets {
    /// Creates a matcher optimized for fuzzy string matching
    pub fn fuzzy_string_matcher() -> Matcher<String, String> {
        MatcherBuilder::<String, String>::new()
            .metric(ExactMatchMetric, 1.0)
            .metric(JaroWinklerSimilarity::default(), 0.8)
            .metric(LevenshteinSimilarity, 0.6)
            .threshold(0.7)
            .option("case_sensitive", "false")
            .build()
    }

    /// Creates a matcher optimized for name matching
    pub fn name_matcher() -> Matcher<String, String> {
        let mut composite = CompositeSimilarity::<String, String>::new("name_matcher", CompositeStrategy::Maximum);
        composite
            .add_metric(SoundexSimilarity::default())
            .add_metric(JaroWinklerSimilarity::default());

        MatcherBuilder::<String, String>::new()
            .metric(ExactMatchMetric, 1.0)
            .metric(composite, 0.8)
            .threshold(0.6)
            .option("case_sensitive", "false")
            .build()
    }

    /// Creates a matcher optimized for address matching
    pub fn address_matcher() -> Matcher<String, String> {
        MatcherBuilder::new()
            .begin_group("token_matching")
            .metric(WordOverlapSimilarity::default(), 0.7)
            .metric(CosineSimilarity::default(), 0.5)
            .end_group()
            .metric(JaroWinklerSimilarity::default(), 0.3)
            .threshold(0.6)
            .option("normalize_addresses", "true")
            .build()
    }

    /// Creates a matcher optimized for numeric proximity
    pub fn numeric_matcher() -> Matcher<f64, f64> {
        // We'd need to implement NumericSimilarity separately
        #[derive(Debug)]
        struct NumericSimilarity {
            max_difference: f64,
        }

        impl SimilarityMetric<f64, f64> for NumericSimilarity {
            fn calculate(&self, query: &f64, candidate: &f64) -> f64 {
                let diff = (query - candidate).abs();
                if diff > self.max_difference {
                    0.0
                } else {
                    1.0 - (diff / self.max_difference)
                }
            }

        }

        MatcherBuilder::new()
            .metric(NumericSimilarity { max_difference: 1000.0 }, 1.0)
            .threshold(0.8)
            .build()
    }

    /// Creates a matcher optimized for document/text similarity
    pub fn document_matcher() -> Matcher<String, String> {
        MatcherBuilder::new()
            .metric(WordOverlapSimilarity::default(), 0.6)
            .metric(CosineSimilarity::new(3), 0.4) // trigrams
            .threshold(0.5)
            .option("normalize_whitespace", "true")
            .option("ignore_case", "true")
            .build()
    }

    /// Creates a customizable fuzzy matcher with sensible defaults
    pub fn customize_fuzzy_matcher() -> MatcherBuilder<String, String> {
        MatcherBuilder::<String, String>::new()
            .metric(ExactMatchMetric, 1.0)
            .metric(JaroWinklerSimilarity::default(), 0.7)
            .metric(LevenshteinSimilarity, 0.5)
            .threshold(0.6)
    }
}

/// Common similarity metrics used across Matchete example files.
/// These metrics demonstrate how to implement the `SimilarityMetric` trait
/// for strings and numbers.
use hashish::HashSet;

/// A simple Levenshtein-like string similarity metric (for demonstration).
/// Computes a score based on the difference in string lengths.
#[derive(Debug)]
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
}

/// Jaccard similarity metric for strings based on character sets.
/// Computes the intersection over union of character sets.
#[derive(Debug)]
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
}

/// Similarity metric for f64 numbers based on absolute difference.
/// Normalizes the difference within a fixed range (100.0).
#[derive(Debug)]
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
}
