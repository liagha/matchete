use matchete::{Matcher, Custom, Similarity};

// Levenshtein metric implementation
struct LevenshteinMetric;

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

// Jaccard metric implementation
struct JaccardMetric;

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
    // Create a matcher with two metrics
    let matcher = Matcher::<String, String>::new()
        .add(LevenshteinMetric, 0.7) // 70% weight
        .add(JaccardMetric, 0.3)     // 30% weight
        .threshold(0.5);

    // Define the query and candidate
    let query = String::from("test");
    let candidate = String::from("tent");

    // Perform detailed analysis
    println!("Detailed Analysis Example");
    println!("========================");
    let analysis = matcher.analyze(&query, &candidate);
    println!("Query: {}", analysis.query);
    println!("Candidate: {}", analysis.candidate);
    println!("Overall score: {:.2}", analysis.score);
    println!("Exact match: {}", analysis.exact);
    println!("Is match: {}", matcher.matches(&query, &candidate));
    println!("Individual metric scores:");
    for (i, score) in analysis.scores.iter().enumerate() {
        println!(
            "  Metric {}:\n    Raw score: {:.2}\n    Weight: {:.2}\n    Weighted score: {:.2}",
            i + 1, score.value, score.weight, score.weighted()
        );
    }
}
