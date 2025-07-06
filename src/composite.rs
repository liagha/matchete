use core::{fmt::Debug, marker::PhantomData};
use crate::Scorer;

#[derive(Debug)]
pub enum Strategy {
    Max,
    Average,
    Fallback(f64),
    Weighted(Vec<f64>),
}

#[derive(Debug)]
pub struct Composite<Query, Item> {
    name: String,
    scorers: Vec<Box<dyn Scorer<Query, Item>>>,
    strategy: Strategy,
    phantom: PhantomData<(Query, Item)>,
}

impl<Query, Item> Composite<Query, Item> {
    pub fn new<N: Into<String>>(name: N, strategy: Strategy) -> Self {
        Self {
            name: name.into(),
            scorers: Vec::new(),
            strategy,
            phantom: PhantomData,
        }
    }

    pub fn add<S: Scorer<Query, Item> + 'static>(mut self, scorer: S) -> Self {
        self.scorers.push(Box::new(scorer));
        self
    }

    pub fn with<S: Scorer<Query, Item> + 'static>(mut self, scorer: S) -> Self {
        self.scorers.push(Box::new(scorer));
        self
    }
}

impl<Query, Item> Scorer<Query, Item> for Composite<Query, Item>
where
    Query: Debug,
    Item: Debug,
{
    fn score(&self, query: &Query, item: &Item) -> f64 {
        if self.scorers.is_empty() {
            return 0.0;
        }

        let scores: Vec<f64> = self.scorers.iter()
            .map(|scorer| scorer.score(query, item))
            .collect();

        match &self.strategy {
            Strategy::Max => {
                scores.iter().fold(0.0, |a, &b| a.max(b))
            }
            Strategy::Average => {
                scores.iter().sum::<f64>() / scores.len() as f64
            }
            Strategy::Fallback(threshold) => {
                scores.iter()
                    .find(|&&s| s >= *threshold)
                    .copied()
                    .unwrap_or(0.0)
            }
            Strategy::Weighted(weights) => {
                let total_weighted: f64 = scores.iter()
                    .enumerate()
                    .map(|(i, &s)| s * weights.get(i).copied().unwrap_or(1.0))
                    .sum();
                let total_weight: f64 = weights.iter().sum();

                if total_weight > 0.0 {
                    total_weighted / total_weight
                } else {
                    0.0
                }
            }
        }
    }

    fn exact(&self, query: &Query, item: &Item) -> bool {
        self.scorers.iter().any(|scorer| scorer.exact(query, item))
    }
}