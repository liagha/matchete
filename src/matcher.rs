use core::fmt::Debug;
use crate::{Scorer, Weight, Detail, Result, Kind, Weighted};

pub struct Matcher<Query, Item> {
    scorers: Vec<Box<dyn Scorer<Query, Item>>>,
    weights: Vec<Weight>,
    threshold: f64,
}

impl<Query, Item> Matcher<Query, Item> {
    pub fn new() -> Self {
        Self {
            scorers: Vec::new(),
            weights: Vec::new(),
            threshold: 0.4,
        }
    }

    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn add<S: Scorer<Query, Item> + 'static, N: Into<String>>(
        mut self,
        scorer: S,
        weight: f64,
        name: N
    ) -> Self {
        self.scorers.push(Box::new(scorer));
        self.weights.push(Weight {
            value: weight,
            name: name.into(),
        });
        self
    }
}

impl<Query, Item> Matcher<Query, Item>
where
    Query: Clone + Debug,
    Item: Clone + Debug,
{
    pub fn score(&self, query: &Query, item: &Item) -> f64 {
        let total_weighted: f64 = self.scorers.iter()
            .zip(&self.weights)
            .map(|(scorer, weight)| scorer.score(query, item) * weight.value)
            .sum();

        let total_weight: f64 = self.weights.iter().map(|w| w.value).sum();

        if total_weight > 0.0 {
            total_weighted / total_weight
        } else {
            0.0
        }
    }

    pub fn exact(&self, query: &Query, item: &Item) -> bool {
        self.scorers.iter().any(|scorer| scorer.exact(query, item))
    }

    pub fn kind(&self, query: &Query, item: &Item) -> Kind {
        if self.exact(query, item) {
            Kind::Exact
        } else if self.score(query, item) >= self.threshold {
            Kind::Similar
        } else {
            Kind::None
        }
    }

    pub fn details(&self, query: &Query, item: &Item) -> Vec<Detail> {
        self.scorers.iter()
            .zip(&self.weights)
            .map(|(scorer, weight)| {
                let score = scorer.score(query, item);
                Detail::new(score, weight.clone())
            })
            .collect()
    }

    pub fn result(&self, query: &Query, item: &Item) -> Result<Query, Item> {
        let details = self.details(query, item);
        let score = self.score(query, item);
        let exact = self.exact(query, item);

        Result {
            query: query.clone(),
            item: item.clone(),
            score,
            exact,
            details,
        }
    }

    pub fn matches(&self, query: &Query, item: &Item) -> bool {
        !matches!(self.kind(query, item), Kind::None)
    }

    pub fn best(&self, query: &Query, items: &[Item]) -> Option<Result<Query, Item>> {
        items.iter()
            .map(|item| self.result(query, item))
            .filter(|result| result.exact || result.score >= self.threshold)
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }

    pub fn find(&self, query: &Query, items: &[Item]) -> Vec<Result<Query, Item>> {
        let mut results: Vec<Result<Query, Item>> = items.iter()
            .map(|item| self.result(query, item))
            .filter(|result| result.exact || result.score >= self.threshold)
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    pub fn limit(&self, query: &Query, items: &[Item], limit: usize) -> Vec<Result<Query, Item>> {
        let mut results = self.find(query, items);
        results.truncate(limit);
        results
    }
}