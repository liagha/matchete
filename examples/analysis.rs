/// Demonstrates detailed match analysis in Matchete, including per-metric scores.
/// Uses multiple string metrics to analyze a query-candidate pair.
///
/// Run this example with:
/// ```bash
/// cargo run --example detailed_analysis
/// ```
use matchete::{Matcher, DetailedMatchResult, MatchType};
use common::{SimpleLevenshteinMetric, JaccardMetric};

fn main() {
    // Create a matcher with two metrics
    let matcher = Matcher::<String, String>::new()
        .with_metric(SimpleLevenshteinMetric, 0.7) // 70% weight
        .with_metric(JaccardMetric, 0.3)          // 30% weight
        .with_threshold(0.5);

    // Define the query and candidate
    let query = String::from("test");
    let candidate = String::from("tent");

    // Perform detailed analysis
    println!("Detailed Analysis Example");
    println!("========================");
    let result: DetailedMatchResult<String, String> = matcher.analyze(&query, &candidate);
    println!("Query: {}", result.query);
    println!("Candidate: {}", result.candidate);
    println!("Overall score: {:.2}", result.score);
    println!("Match type: {:?}", result.match_type);
    println!("Is match: {}", result.is_match);
    println!("Metric scores:");
    for score in result.metric_scores {
        println!(
            "  {}:\n    Raw score: {:.2}\n    Weighted score: {:.2}",
            score.id, score.raw_score, score.weighted_score
        );
    }
}

// Ensure the common module is accessible
mod common;