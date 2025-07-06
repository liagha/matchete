/// Demonstrates basic string matching using a single similarity metric in Matchete.
/// Uses a simple Levenshtein-like metric to find the best match for a query string.
///
/// Run this example with:
/// ```bash
/// cargo run --example string
/// ```
use matchete::{
    Matcher,
    SimpleLevenshteinMetric,
};

fn main() {
    // Create a matcher with a single metric and a threshold
    let matcher = Matcher::<String, String>::new()
        .with_metric(SimpleLevenshteinMetric, 1.0)
        .with_threshold(0.6);

    // Define the query and candidate strings
    let query = String::from("hello");
    let candidates = vec![
        String::from("hello"),
        String::from("helo"),
        String::from("world"),
    ];

    // Find the best match
    println!("Basic String Matching Example");
    println!("============================");
    if let Some(result) = matcher.find_best_match(&query, &candidates) {
        println!(
            "Best match found:\n  Score: {:.2}\n  Candidate: {}\n  Match type: {:?}",
            result.score, result.candidate, result.match_type
        );
    } else {
        println!("No match found above threshold");
    }
}