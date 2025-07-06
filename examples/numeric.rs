/// Demonstrates matching f64 numbers using a custom similarity metric in Matchete.
/// Uses a metric based on the absolute difference between numbers.
///
/// Run this example with:
/// ```bash
/// cargo run --example numeric_matching
/// ```
use matchete::{
    Matcher,
    NumericSimilarity
};

fn main() {
    // Create a matcher for f64 numbers
    let matcher = Matcher::<f64, f64>::new()
        .with_metric(NumericSimilarity, 1.0)
        .with_threshold(0.8);

    // Define the query and candidate numbers
    let query = 42.0;
    let candidates = vec![40.0, 45.0, 100.0];

    // Find all matches
    println!("Numeric Matching Example");
    println!("=======================");
    let matches = matcher.find_matches(&query, &candidates, 0);
    println!("Matches found: {}", matches.len());
    for (i, m) in matches.iter().enumerate() {
        println!(
            "Match {}:\n  Score: {:.2}\n  Candidate: {}\n  Match type: {:?}",
            i + 1, m.score, m.candidate, m.match_type
        );
    }
}