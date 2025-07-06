use matchete::{Matcher, Scorer};

#[derive(Debug)]
struct LevenshteinScorer;

impl Scorer<String, String> for LevenshteinScorer {
    fn score(&self, query: &String, item: &String) -> f64 {
        let distance = levenshtein_distance(query, item);
        let max_len = query.len().max(item.len());
        if max_len == 0 { 1.0 } else { 1.0 - (distance as f64 / max_len as f64) }
    }

    fn exact(&self, query: &String, item: &String) -> bool {
        query == item
    }
}

#[derive(Debug)]
struct JaccardScorer;

impl Scorer<String, String> for JaccardScorer {
    fn score(&self, query: &String, item: &String) -> f64 {
        let query_chars: std::collections::HashSet<char> = query.chars().collect();
        let item_chars: std::collections::HashSet<char> = item.chars().collect();

        let intersection = query_chars.intersection(&item_chars).count();
        let union = query_chars.union(&item_chars).count();

        if union == 0 { 1.0 } else { intersection as f64 / union as f64 }
    }

    fn exact(&self, query: &String, item: &String) -> bool {
        query == item
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
        .add(LevenshteinScorer, 0.7, "levenshtein")
        .add(JaccardScorer, 0.3, "jaccard")
        .threshold(0.5);

    let query = String::from("test");
    let item = String::from("tent");

    println!("Detailed Analysis Example");
    println!("========================");

    let result = matcher.result(&query, &item);
    println!("Query: {}", result.query);
    println!("Item: {}", result.item);
    println!("Overall score: {:.2}", result.score);
    println!("Exact match: {}", result.exact);
    println!("Is match: {}", matcher.matches(&query, &item));
    println!("Individual scorer details:");

    for (i, detail) in result.details.iter().enumerate() {
        println!("  Scorer {}: {}", i + 1, detail.weight.name);
        println!("    Raw score: {:.2}", detail.score);
        println!("    Weight: {:.2}", detail.weight.value);
        println!("    Weighted score: {:.2}", detail.score * detail.weight.value);
    }
}
