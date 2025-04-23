/// Demonstrates combining multiple similarity metrics using MatcherBuilder in Matchete.
/// Uses Levenshtein and Jaccard metrics with weights and a conditional threshold.
///
/// Run this example with:
/// ```bash
/// cargo run --example multimetric
/// ```
use matchete::{MatcherBuilder, MatchType};
use common::{SimpleLevenshteinMetric, JaccardMetric};

fn main() {
    // Build a matcher with multiple metrics and a conditional threshold
    let matcher = MatcherBuilder::<String, String>::new()
        .begin_group("text_metrics")
        .metric(SimpleLevenshteinMetric, 0.6) // 60% weight
        .metric(JaccardMetric, 0.4)          // 40% weight
        .end_group()
        .threshold(0.5)
        .conditional_threshold(|query: &String| query.len() > 10, 0.7)
        .option("case_sensitive", "false")
        .build();

    // Define the query and candidate strings
    let query = String::from("hello world");
    let candidates = vec![
        String::from("hello"),
        String::from("hello there"),
        String::from("world"),
    ];

    // Find matches with a limit of 2
    println!("Multiple Metrics Example");
    println!("=======================");
    let matches = matcher.find_matches(&query, &candidates, 2);
    println!("Matches found: {}", matches.len());
    for (i, m) in matches.iter().enumerate() {
        println!(
            "Match {}:\n  Score: {:.2}\n  Candidate: {}\n  Match type: {:?}",
            i + 1, m.score, m.candidate, m.match_type
        );
    }
}

// Ensure the common module is accessible
mod common;