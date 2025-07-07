use {
    core::{
        fmt::Debug,
    },
    matchete::{Assessor, Resemblance},
};

#[derive(Debug)]
struct NumericResembler;

impl Resemblance<f64, f64> for NumericResembler {
    fn resemblance(&self, query: &f64, candidate: &f64) -> f64 {
        let diff = (query - candidate).abs();
        let max_val = query.abs().max(candidate.abs());
        if max_val == 0.0 { 1.0 } else { 1.0 - (diff / (max_val + 1.0)) }
    }

    fn perfect(&self, query: &f64, candidate: &f64) -> bool {
        (query - candidate).abs() < f64::EPSILON
    }
}

fn main() {
    let assessor = Assessor::<f64, f64>::new()
        .dimension(NumericResembler, 1.0)
        .floor(0.8);

    let query = 42.0;
    let candidates = vec![40.0, 45.0, 100.0];

    println!("Numeric Matching Example");
    println!("=======================");

    let shortlist = assessor.shortlist(&query, &candidates);
    println!("Viable matches found: {}", shortlist.len());

    for (i, verdict) in shortlist.iter().enumerate() {
        println!("Match {}:", i + 1);
        println!("  Resemblance: {:.2}", verdict.resemblance);
        println!("  Candidate: {}", verdict.candidate);
        println!("  Perfect: {}", verdict.perfect);
    }

    // Show champion example
    println!("\nChampion Selection");
    println!("=================");

    if let Some(champion) = assessor.champion(&query, &candidates) {
        println!("Best candidate: {} (resemblance: {:.2})", champion.candidate, champion.resemblance);
    } else {
        println!("No viable candidates found");
    }

    // Show all candidates with their resemblance scores
    println!("\nAll Candidates Analysis");
    println!("======================");

    for candidate in &candidates {
        let profile = assessor.profile(&query, candidate);
        println!("Candidate {}: resemblance={:.2}",
                 candidate, profile.resemblance);
    }
}