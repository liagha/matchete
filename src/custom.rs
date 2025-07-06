#![allow(dead_code)]

use {
    core::{
        fmt::Debug,
        marker::PhantomData,
    },
    crate::{
        SimilarityMetric,
    }
};

/// Closure-based similarity metric
#[derive(Debug)]
pub struct ClosureMetric<Q: Debug, C: Debug, F: Debug>
where
    F: Fn(&Q, &C) -> f64, Q: Debug, C: Debug, F: Debug,
{
    id: String,
    calculate_fn: F,
    _phantom: PhantomData<(Q, C)>,
}

impl<Q, C, F> ClosureMetric<Q, C, F>
where
    F: Fn(&Q, &C) -> f64, Q: Debug, C: Debug, F: Debug,
{
    pub fn new<S: Into<String>>(id: S, calculate_fn: F) -> Self {
        Self {
            id: id.into(),
            calculate_fn,
            _phantom: PhantomData,
        }
    }
}

impl<Q, C, F> SimilarityMetric<Q, C> for ClosureMetric<Q, C, F>
where
    F: Fn(&Q, &C) -> f64, Q: Debug, C: Debug, F: Debug,
{
    fn calculate(&self, query: &Q, candidate: &C) -> f64 {
        (self.calculate_fn)(query, candidate)
    }
}