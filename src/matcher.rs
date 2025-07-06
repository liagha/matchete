use {
    core::{
        fmt::Debug,
        marker::PhantomData,
    },
    hashish::HashMap,
};

pub trait Similarity<Q, C> {
    fn score(&self, query: &Q, candidate: &C) -> f64;
    fn exact(&self, query: &Q, candidate: &C) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct Match<Q, C> {
    pub query: Q,
    pub candidate: C,
    pub score: f64,
    pub exact: bool,
}

#[derive(Debug, Clone)]
pub struct Score {
    pub value: f64,
    pub weight: f64,
}

impl Score {
    pub fn weighted(&self) -> f64 {
        self.value * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct Analysis<Q, C> {
    pub query: Q,
    pub candidate: C,
    pub score: f64,
    pub exact: bool,
    pub scores: Vec<Score>,
}

pub struct Weighted<Q, C, M> {
    metric: M,
    weight: f64,
    _phantom: PhantomData<(Q, C)>,
}

impl<Q, C, M> Weighted<Q, C, M> {
    pub fn new(metric: M, weight: f64) -> Self {
        Self {
            metric,
            weight,
            _phantom: PhantomData,
        }
    }
}

impl<Q, C, M> Similarity<Q, C> for Weighted<Q, C, M>
where
    M: Similarity<Q, C>,
{
    fn score(&self, query: &Q, candidate: &C) -> f64 {
        self.metric.score(query, candidate) * self.weight
    }

    fn exact(&self, query: &Q, candidate: &C) -> bool {
        self.metric.exact(query, candidate)
    }
}

pub struct Custom<Q, C, F> {
    function: F,
    _phantom: PhantomData<(Q, C)>,
}

impl<Q, C, F> Custom<Q, C, F>
where
    F: Fn(&Q, &C) -> f64,
{
    pub fn new(function: F) -> Self {
        Self {
            function,
            _phantom: PhantomData,
        }
    }
}

impl<Q, C, F> Similarity<Q, C> for Custom<Q, C, F>
where
    F: Fn(&Q, &C) -> f64,
{
    fn score(&self, query: &Q, candidate: &C) -> f64 {
        (self.function)(query, candidate)
    }
}

pub enum Strategy {
    Maximum,
    Average,
    Fallback(f64),
    Weighted(Vec<f64>),
}

pub struct Composite<Q, C> {
    metrics: Vec<Box<dyn Similarity<Q, C>>>,
    strategy: Strategy,
    _phantom: PhantomData<(Q, C)>,
}

impl<Q, C> Composite<Q, C> {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            metrics: Vec::new(),
            strategy,
            _phantom: PhantomData,
        }
    }

    pub fn add<M: Similarity<Q, C> + 'static>(mut self, metric: M) -> Self {
        self.metrics.push(Box::new(metric));
        self
    }
}

impl<Q, C> Similarity<Q, C> for Composite<Q, C> {
    fn score(&self, query: &Q, candidate: &C) -> f64 {
        if self.metrics.is_empty() {
            return 0.0;
        }

        let scores: Vec<f64> = self.metrics.iter()
            .map(|m| m.score(query, candidate))
            .collect();

        match &self.strategy {
            Strategy::Maximum => {
                scores.iter().fold(0.0, |a, &b| a.max(b))
            },
            Strategy::Average => {
                scores.iter().sum::<f64>() / scores.len() as f64
            },
            Strategy::Fallback(threshold) => {
                scores.iter()
                    .find(|&&s| s >= *threshold)
                    .copied()
                    .unwrap_or(0.0)
            },
            Strategy::Weighted(weights) => {
                let total_weighted: f64 = scores.iter().enumerate()
                    .map(|(i, &s)| s * weights.get(i).copied().unwrap_or(1.0))
                    .sum();
                let total_weights: f64 = weights.iter().sum();
                if total_weights > 0.0 {
                    total_weighted / total_weights
                } else {
                    0.0
                }
            }
        }
    }

    fn exact(&self, query: &Q, candidate: &C) -> bool {
        self.metrics.iter().any(|m| m.exact(query, candidate))
    }
}

pub struct Matcher<Q, C> {
    metrics: Vec<Box<dyn Similarity<Q, C>>>,
    weights: Vec<f64>,
    threshold: f64,
}

impl<Q, C> Default for Matcher<Q, C> {
    fn default() -> Self {
        Self {
            metrics: Vec::new(),
            weights: Vec::new(),
            threshold: 0.4,
        }
    }
}

impl<Q: Clone + Debug, C: Clone + Debug> Matcher<Q, C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<M: Similarity<Q, C> + 'static>(mut self, metric: M, weight: f64) -> Self {
        self.metrics.push(Box::new(metric));
        self.weights.push(weight);
        self
    }

    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn analyze(&self, query: &Q, candidate: &C) -> Analysis<Q, C> {
        let exact = self.metrics.iter().any(|m| m.exact(query, candidate));

        let scores: Vec<Score> = self.metrics.iter()
            .zip(&self.weights)
            .map(|(m, &w)| Score {
                value: m.score(query, candidate),
                weight: w,
            })
            .collect();

        let total_weighted: f64 = scores.iter().map(|s| s.weighted()).sum();
        let total_weight: f64 = self.weights.iter().sum();

        let overall_score = if total_weight > 0.0 {
            total_weighted / total_weight
        } else {
            0.0
        };

        Analysis {
            query: query.clone(),
            candidate: candidate.clone(),
            score: overall_score,
            exact,
            scores,
        }
    }

    pub fn score(&self, query: &Q, candidate: &C) -> f64 {
        self.analyze(query, candidate).score
    }

    pub fn matches(&self, query: &Q, candidate: &C) -> bool {
        let analysis = self.analyze(query, candidate);
        analysis.exact || analysis.score >= self.threshold
    }

    pub fn best(&self, query: &Q, candidates: &[C]) -> Option<Match<Q, C>> {
        candidates.iter()
            .map(|c| {
                let analysis = self.analyze(query, c);
                Match {
                    query: query.clone(),
                    candidate: c.clone(),
                    score: analysis.score,
                    exact: analysis.exact,
                }
            })
            .filter(|m| m.exact || m.score >= self.threshold)
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }

    pub fn find(&self, query: &Q, candidates: &[C]) -> Vec<Match<Q, C>> {
        let mut matches: Vec<Match<Q, C>> = candidates.iter()
            .map(|c| {
                let analysis = self.analyze(query, c);
                Match {
                    query: query.clone(),
                    candidate: c.clone(),
                    score: analysis.score,
                    exact: analysis.exact,
                }
            })
            .filter(|m| m.exact || m.score >= self.threshold)
            .collect();

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        matches
    }

    pub fn find_limit(&self, query: &Q, candidates: &[C], limit: usize) -> Vec<Match<Q, C>> {
        let mut matches = self.find(query, candidates);
        if matches.len() > limit {
            matches.truncate(limit);
        }
        matches
    }
}

pub struct MultiMatcher<Q, C> {
    matchers: Vec<Matcher<Q, C>>,
    threshold: f64,
}

impl<Q, C> Default for MultiMatcher<Q, C> {
    fn default() -> Self {
        Self {
            matchers: Vec::new(),
            threshold: 0.4,
        }
    }
}

impl<Q: Clone + Debug, C: Clone + Debug + PartialEq> MultiMatcher<Q, C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(mut self, matcher: Matcher<Q, C>) -> Self {
        self.matchers.push(matcher);
        self
    }

    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn best(&self, query: &Q, candidates: &[C]) -> Option<Match<Q, C>> {
        self.matchers.iter()
            .filter_map(|m| m.best(query, candidates))
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }

    pub fn find(&self, query: &Q, candidates: &[C]) -> Vec<Match<Q, C>> {
        let mut all_matches = Vec::new();

        for matcher in &self.matchers {
            all_matches.extend(matcher.find(query, candidates));
        }

        all_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_matches.dedup_by(|a, b| a.candidate == b.candidate);
        all_matches
    }

    pub fn find_limit(&self, query: &Q, candidates: &[C], limit: usize) -> Vec<Match<Q, C>> {
        let mut matches = self.find(query, candidates);
        if matches.len() > limit {
            matches.truncate(limit);
        }
        matches
    }
}

pub struct Builder<Q, C> {
    matcher: Matcher<Q, C>,
}

impl<Q: Clone + Debug, C: Clone + Debug> Builder<Q, C> {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(),
        }
    }

    pub fn metric<M: Similarity<Q, C> + 'static>(mut self, metric: M, weight: f64) -> Self {
        self.matcher = self.matcher.add(metric, weight);
        self
    }

    pub fn threshold(mut self, threshold: f64) -> Self {
        self.matcher = self.matcher.threshold(threshold);
        self
    }

    pub fn build(self) -> Matcher<Q, C> {
        self.matcher
    }
}