use std::fmt::{Debug, Formatter};
use matchete::{Matcher, Scorer};

#[derive(Debug)]
struct LevenshteinScorer;

impl Scorer<String, String> for LevenshteinScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        let distance = levenshtein_distance(query, candidate);
        let max_len = query.len().max(candidate.len());
        if max_len == 0 { 1.0 } else { 1.0 - (distance as f64 / max_len as f64) }
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        query == candidate
    }
}

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

fn main() {
    let matcher = Matcher::<String, String>::new()
        .add(LevenshteinScorer, 1.0, "levenshtein")
        .threshold(0.6);

    let query = String::from("hello");
    let candidates = vec![
        String::from("hello"),
        String::from("helo"),
        String::from("world"),
    ];

    println!("Basic String Matching Example");
    println!("============================");

    if let Some(result) = matcher.best(&query, &candidates) {
        println!("Best match found:");
        println!("  Score: {:.2}", result.score);
        println!("  Candidate: {}", result.candidate);
        println!("  Exact: {}", result.exact);
    } else {
        println!("No match found above threshold");
    }
}