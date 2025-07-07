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

#[derive(Debug)]
struct JaccardResembler;

impl Resemblance<String, String> for JaccardResembler {
    fn resemblance(&self, query: &String, candidate: &String) -> f64 {
        let query_chars: std::collections::HashSet<char> = query.chars().collect();
        let item_chars: std::collections::HashSet<char> = candidate.chars().collect();

        let intersection = query_chars.intersection(&item_chars).count();
        let union = query_chars.union(&item_chars).count();

        if union == 0 { 1.0 } else { intersection as f64 / union as f64 }
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
        .dimension(LevenshteinResembler, 0.7)
        .dimension(JaccardResembler, 0.3)
        .floor(0.5);

    let query = String::from("test");
    let candidate = String::from("tent");

    println!("Detailed Analysis Example");
    println!("========================");

    let profile = assessor.profile(&query, &candidate);
    println!("Query: {}", profile.query);
    println!("Candidate: {}", profile.candidate);
    println!("Overall resemblance: {:.2}", profile.resemblance);
    println!("Perfect match: {}", profile.perfect);
    println!("Is viable: {}", assessor.viable(&query, &candidate));
    println!("Disposition: {:?}", assessor.disposition(&query, &candidate));
    println!("Individual resembler facets:");

    // Get resembler names using Debug trait
    let resembler_names: Vec<String> = assessor.dimensions.iter()
        .map(|resembler| format!("{:?}", resembler))
        .collect();

    // Example of finding best match from multiple candidates
    println!("\nBest Match Example");
    println!("==================");

    let candidates = vec![
        String::from("tent"),
        String::from("test"),
        String::from("toast"),
        String::from("rest"),
    ];

    if let Some(champion) = assessor.champion(&query, &candidates) {
        println!("Champion: {} (resemblance: {:.2})", champion.candidate, champion.resemblance);
    }

    // Example of getting shortlist
    println!("\nShortlist Example");
    println!("=================");

    let shortlist = assessor.shortlist(&query, &candidates);
    for (i, profile) in shortlist.iter().enumerate() {
        println!("{}. {} (resemblance: {:.2})", i + 1, profile.candidate, profile.resemblance);
    }
}