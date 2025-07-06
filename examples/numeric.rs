use matchete::{Matcher, Custom};

fn main() {
    // Create a numeric similarity metric
    let numeric_metric = Custom::new(|query: &f64, candidate: &f64| {
        let diff = (query - candidate).abs();
        let max_val = query.abs().max(candidate.abs());
        if max_val == 0.0 { 1.0 } else { 1.0 - (diff / (max_val + 1.0)) }
    });

    // Create a matcher for f64 numbers
    let matcher = Matcher::<f64, f64>::new()
        .add(numeric_metric, 1.0)
        .threshold(0.8);

    // Define the query and candidate numbers
    let query = 42.0;
    let candidates = vec![40.0, 45.0, 100.0];

    // Find all matches
    println!("Numeric Matching Example");
    println!("=======================");
    let matches = matcher.find(&query, &candidates);
    println!("Matches found: {}", matches.len());
    for (i, m) in matches.iter().enumerate() {
        println!(
            "Match {}:\n  Score: {:.2}\n  Candidate: {}\n  Exact: {}",
            i + 1, m.score, m.candidate, m.exact
        );
    }
}