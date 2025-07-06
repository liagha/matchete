use std::fmt::{Debug, Formatter};
use matchete::{Matcher, Scorer};

#[derive(Debug)]
struct NumericScorer;

impl Scorer<f64, f64> for NumericScorer {
    fn score(&self, query: &f64, item: &f64) -> f64 {
        let diff = (query - item).abs();
        let max_val = query.abs().max(item.abs());
        if max_val == 0.0 { 1.0 } else { 1.0 - (diff / (max_val + 1.0)) }
    }

    fn exact(&self, query: &f64, item: &f64) -> bool {
        (query - item).abs() < f64::EPSILON
    }
}

fn main() {
    let matcher = Matcher::<f64, f64>::new()
        .add(NumericScorer, 1.0, "numeric")
        .threshold(0.8);

    let query = 42.0;
    let candidates = vec![40.0, 45.0, 100.0];

    println!("Numeric Matching Example");
    println!("=======================");

    let matches = matcher.find(&query, &candidates);
    println!("Matches found: {}", matches.len());

    for (i, result) in matches.iter().enumerate() {
        println!("Match {}:", i + 1);
        println!("  Score: {:.2}", result.score);
        println!("  Item: {}", result.item);
        println!("  Exact: {}", result.exact);
    }
}