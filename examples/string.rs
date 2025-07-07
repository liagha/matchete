use std::fmt::{Debug};
use matchete::{Assessor, Resemblance};

#[derive(Debug)]
struct LevenshteinResembler;

impl Resemblance<String, String> for LevenshteinResembler {
    fn resemblance(&self, query: &String, candidate: &String) -> f64 {
        let distance = levenshtein_distance(query, candidate);
        let max_len = query.len().max(candidate.len());
        if max_len == 0 { 1.0 } else { 1.0 - (distance as f64 / max_len as f64) }
    }

    fn perfect(&self, query: &String, candidate: &String) -> bool {
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
    let assessor = Assessor::<String, String>::new()
        .dimension(LevenshteinResembler, 1.0)
        .floor(0.6);

    let query = String::from("hello");
    let candidates = vec![
        String::from("hello"),
        String::from("helo"),
        String::from("world"),
    ];

    println!("Basic String Matching Example");
    println!("============================");

    if let Some(verdict) = assessor.champion(&query, &candidates) {
        println!("Champion found:");
        println!("  Resemblance: {:.2}", verdict.resemblance);
        println!("  Candidate: {}", verdict.candidate);
        println!("  Perfect: {}", verdict.perfect);
    } else {
        println!("No viable candidate found above floor threshold");
    }

    // Show all candidates for comparison
    println!("\nAll Candidates Analysis");
    println!("======================");

    for candidate in &candidates {
        let profile = assessor.profile(&query, candidate);
        println!("'{}': resemblance={:.2}",
                 candidate, profile.resemblance);
    }
}