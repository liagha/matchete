use core::fmt::Debug;
use crate::{Scorer, Weight, Detail, Product, Kind};

pub struct Matcher<Query, Candidate> {
    scorers: Vec<Box<dyn Scorer<Query, Candidate>>>,
    weights: Vec<Weight>,
    threshold: f64,
}

impl<Query, Candidate> Matcher<Query, Candidate> {
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

    pub fn add<S: Scorer<Query, Candidate> + 'static, N: Into<String>>(
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

impl<Query, Candidate> Matcher<Query, Candidate>
where
    Query: Clone + Debug,
    Candidate: Clone + Debug,
{
    pub fn score(&self, query: &Query, candidate: &Candidate) -> f64 {
        let total_weighted: f64 = self.scorers.iter()
            .zip(&self.weights)
            .map(|(scorer, weight)| scorer.score(query, candidate) * weight.value)
            .sum();

        let total_weight: f64 = self.weights.iter().map(|w| w.value).sum();

        if total_weight > 0.0 {
            total_weighted / total_weight
        } else {
            0.0
        }
    }

    pub fn exact(&self, query: &Query, candidate: &Candidate) -> bool {
        self.scorers.iter().any(|scorer| scorer.exact(query, candidate))
    }

    pub fn kind(&self, query: &Query, candidate: &Candidate) -> Kind {
        if self.exact(query, candidate) {
            Kind::Exact
        } else if self.score(query, candidate) >= self.threshold {
            Kind::Similar
        } else {
            Kind::None
        }
    }

    pub fn details(&self, query: &Query, candidate: &Candidate) -> Vec<Detail> {
        self.scorers.iter()
            .zip(&self.weights)
            .map(|(scorer, weight)| {
                let score = scorer.score(query, candidate);
                Detail::new(score, weight.clone())
            })
            .collect()
    }

    pub fn result(&self, query: &Query, candidate: &Candidate) -> Product<Query, Candidate> {
        let details = self.details(query, candidate);
        let score = self.score(query, candidate);
        let exact = self.exact(query, candidate);

        Product {
            query: query.clone(),
            candidate: candidate.clone(),
            score,
            exact,
            details,
        }
    }

    pub fn matches(&self, query: &Query, candidate: &Candidate) -> bool {
        !matches!(self.kind(query, candidate), Kind::None)
    }

    pub fn best(&self, query: &Query, items: &[Candidate]) -> Option<Product<Query, Candidate>> {
        items.iter()
            .map(|candidate| self.result(query, candidate))
            .filter(|result| result.exact || result.score >= self.threshold)
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }

    pub fn find(&self, query: &Query, items: &[Candidate]) -> Vec<Product<Query, Candidate>> {
        let mut results: Vec<Product<Query, Candidate>> = items.iter()
            .map(|candidate| self.result(query, candidate))
            .filter(|result| result.exact || result.score >= self.threshold)
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    pub fn limit(&self, query: &Query, items: &[Candidate], limit: usize) -> Vec<Product<Query, Candidate>> {
        let mut results = self.find(query, items);
        results.truncate(limit);
        results
    }
}