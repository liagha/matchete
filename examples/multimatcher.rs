use matchete::{Matcher, MultiMatcher, Custom, Similarity};

// Reuse the metrics from analysis.rs
struct LevenshteinMetric;
struct JaccardMetric;

impl Similarity<String, String> for LevenshteinMetric {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let distance = levenshtein_distance(query, candidate);
        let max_len = query.len().max(candidate.len());
        if max_len == 0 { 1.0 } else { 1.0 - (distance as f64 / max_len as f64) }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

impl Similarity<String, String> for JaccardMetric {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let query_chars: std::collections::HashSet<char> = query.chars().collect();
        let candidate_chars: std::collections::HashSet<char> = candidate.chars().collect();

        let intersection = query_chars.intersection(&candidate_chars).count();
        let union = query_chars.union(&candidate_chars).count();

        if union == 0 { 1.0 } else { intersection as f64 / union as f64 }
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    // Same implementation as in analysis.rs
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

fn main() {
    // Create two matchers with different metrics and thresholds
    let matcher1 = Matcher::<String, String>::new()
        .add(LevenshteinMetric, 1.0)
        .threshold(0.6);

    let matcher2 = Matcher::<String, String>::new()
        .add(JaccardMetric, 1.0)
        .threshold(0.4);

    // Combine them into a MultiMatcher
    let multi_matcher = MultiMatcher::<String, String>::new()
        .add(matcher1)
        .add(matcher2)
        .threshold(0.5);

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
    let matches = multi_matcher.find_limit(&query, &candidates, 2);
    println!("Matches found: {}", matches.len());
    for (i, m) in matches.iter().enumerate() {
        println!(
            "Match {}:\n  Score: {:.2}\n  Candidate: {}\n  Exact: {}",
            i + 1, m.score, m.candidate, m.exact
        );
    }
}