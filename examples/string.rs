use matchete::{Matcher, Custom};

// Simple Levenshtein-like metric implementation
struct SimpleLevenshteinMetric;

impl SimpleLevenshteinMetric {
    fn levenshtein_distance(a: &str, b: &str) -> usize {
        let len_a = a.len();
        let len_b = b.len();

        if len_a == 0 { return len_b; }
        if len_b == 0 { return len_a; }

        let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

        for i in 0..=len_a { matrix[i][0] = i; }
        for j in 0..=len_b { matrix[0][j] = j; }

        for i in 1..=len_a {
            for j in 1..=len_b {
                let cost = if a.chars().nth(i - 1) == b.chars().nth(j - 1) { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len_a][len_b]
    }
}

fn main() {
    // Create a custom similarity metric for strings
    let levenshtein_metric = Custom::new(|query: &String, candidate: &String| {
        let distance = SimpleLevenshteinMetric::levenshtein_distance(query, candidate);
        let max_len = query.len().max(candidate.len());
        if max_len == 0 { 1.0 } else { 1.0 - (distance as f64 / max_len as f64) }
    });

    // Create a matcher with the custom metric
    let matcher = Matcher::<String, String>::new()
        .add(levenshtein_metric, 1.0)
        .threshold(0.6);

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
    if let Some(result) = matcher.best(&query, &candidates) {
        println!(
            "Best match found:\n  Score: {:.2}\n  Candidate: {}\n  Exact: {}",
            result.score, result.candidate, result.exact
        );
    } else {
        println!("No match found above threshold");
    }
}