use core::fmt::Debug;
use crate::MatchType;

/// Trait for any similarity metric that can compare query and candidate items
pub trait SimilarityMetric<Q, C>: Debug {
    fn calculate(&self, query: &Q, candidate: &C) -> f64;

    fn is_exact_match(&self, query: &Q, candidate: &C) -> bool {
        self.calculate(query, candidate) >= 0.9999
    }

    fn match_type(&self, query: &Q, candidate: &C) -> Option<MatchType> {
        let score = self.calculate(query, candidate);

        if self.is_exact_match(query, candidate) {
            Some(MatchType::Exact)
        } else if score > 0.0 {
            let id = format!("{:?}", self);
            Some(MatchType::Similar(id))
        } else {
            None
        }
    }
}

/// A weighted metric with its associated weight
pub struct WeightedMetric<Q: Debug, C: Debug> {
    pub metric: Box<dyn SimilarityMetric<Q, C>>,
    pub weight: f64,
}

impl<Q: Debug, C: Debug> WeightedMetric<Q, C> {
    pub fn new<M: SimilarityMetric<Q, C> + 'static>(metric: M, weight: f64) -> Self {
        Self {
            metric: Box::new(metric),
            weight,
        }
    }
}