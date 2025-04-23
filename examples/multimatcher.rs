/// Demonstrates combining multiple matchers using MultiMatcher in Matchete.
/// Uses two separate matchers with different metrics to find matches.
///
/// Run this example with:
/// ```bash
/// cargo run --example multimatcher
/// ```
use matchete::{Matcher, MultiMatcher, MatchType};
use common::{SimpleLevenshteinMetric, JaccardMetric};

fn main() {
    // Create two matchers with different metrics and thresholds
    let matcher1 = Matcher::<String, String>::new()
        .with_metric(SimpleLevenshteinMetric, 1.0)
        .with_threshold(0.6);

    let matcher2 = Matcher::<String, String>::new()
        .with_metric(JaccardMetric, 1.0)
        .with_threshold(0.4);

    // Combine them into a MultiMatcher
    let multi_matcher = MultiMatcher::<String, String>::new()
        .with_matcher(matcher1)
        .with_matcher(matcher2)
        .with_threshold(0.5);

    // Define the query and candidate strings
    let query = String::from("example");
    let candidates = vec![
        String::from("example"),
        String::from("examp"),
        String::from("test"),
    ];

    // Find matches with a limit of 2
    println!("MultiMatcher Example");
    println!("===================");
    let matches = multi_matcher.find_matches(&query, &candidates, 2);
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