use {
    core::{
        fmt::Debug,
    },
};

pub trait Resemblance<Query, Candidate>: Debug {
    fn resemblance(&self, query: &Query, candidate: &Candidate) -> f64;

    fn perfect(&self, query: &Query, candidate: &Candidate) -> bool {
        self.resemblance(query, candidate) >= 0.9999
    }
}

#[derive(Debug, Clone)]
pub struct Influence {
    pub magnitude: f64,
}

impl Influence {
    pub fn new(magnitude: f64) -> Self {
        Self { magnitude }
    }
}

#[derive(Debug, Clone)]
pub struct Facet {
    pub resemblance: f64,
    pub influence: Influence,
    pub contribution: f64,
}

impl Facet {
    pub fn new(resemblance: f64, influence: Influence) -> Self {
        let contribution = resemblance * influence.magnitude;
        Self { resemblance, influence, contribution }
    }
}

#[derive(Debug, Clone)]
pub struct Verdict<Query, Candidate> {
    pub query: Query,
    pub candidate: Candidate,
    pub resemblance: f64,
    pub perfect: bool,
    pub facets: Vec<Facet>,
}

#[derive(Debug, Clone)]
pub enum Disposition {
    Perfect,
    Adequate,
    Insufficient,
}

pub struct Assessor<Query, Candidate> {
    pub resemblers: Vec<Box<dyn Resemblance<Query, Candidate>>>,
    pub influences: Vec<Influence>,
    pub floor: f64,
}

impl<Query, Candidate> Assessor<Query, Candidate> {
    pub fn new() -> Self {
        Self {
            resemblers: Vec::new(),
            influences: Vec::new(),
            floor: 0.4,
        }
    }

    pub fn floor(mut self, floor: f64) -> Self {
        self.floor = floor;
        self
    }

    pub fn with<R: Resemblance<Query, Candidate> + 'static>(
        mut self,
        resembler: R,
        magnitude: f64,
    ) -> Self {
        self.resemblers.push(Box::new(resembler));
        self.influences.push(Influence::new(magnitude));
        self
    }
}

impl<Query, Candidate> Assessor<Query, Candidate>
where
    Query: Clone + Debug,
    Candidate: Clone + Debug,
{
    pub fn resemblance(&self, query: &Query, candidate: &Candidate) -> f64 {
        let total_contribution: f64 = self.resemblers.iter()
            .zip(&self.influences)
            .map(|(resembler, influence)| resembler.resemblance(query, candidate) * influence.magnitude)
            .sum();

        let total_magnitude: f64 = self.influences.iter().map(|i| i.magnitude).sum();

        if total_magnitude > 0.0 {
            total_contribution / total_magnitude
        } else {
            0.0
        }
    }

    pub fn perfect(&self, query: &Query, candidate: &Candidate) -> bool {
        self.resemblers.iter().any(|resembler| resembler.perfect(query, candidate))
    }

    pub fn disposition(&self, query: &Query, candidate: &Candidate) -> Disposition {
        if self.perfect(query, candidate) {
            Disposition::Perfect
        } else if self.resemblance(query, candidate) >= self.floor {
            Disposition::Adequate
        } else {
            Disposition::Insufficient
        }
    }

    pub fn facets(&self, query: &Query, candidate: &Candidate) -> Vec<Facet> {
        self.resemblers.iter()
            .zip(&self.influences)
            .map(|(resembler, influence)| {
                let resemblance = resembler.resemblance(query, candidate);
                Facet::new(resemblance, influence.clone())
            })
            .collect()
    }

    pub fn verdict(&self, query: &Query, candidate: &Candidate) -> Verdict<Query, Candidate> {
        let facets = self.facets(query, candidate);
        let resemblance = self.resemblance(query, candidate);
        let perfect = self.perfect(query, candidate);

        Verdict {
            query: query.clone(),
            candidate: candidate.clone(),
            resemblance,
            perfect,
            facets,
        }
    }

    pub fn viable(&self, query: &Query, candidate: &Candidate) -> bool {
        !matches!(self.disposition(query, candidate), Disposition::Insufficient)
    }

    pub fn champion(&self, query: &Query, candidates: &[Candidate]) -> Option<Verdict<Query, Candidate>> {
        candidates.iter()
            .map(|candidate| self.verdict(query, candidate))
            .filter(|verdict| verdict.perfect || verdict.resemblance >= self.floor)
            .max_by(|a, b| a.resemblance.partial_cmp(&b.resemblance).unwrap())
    }

    pub fn shortlist(&self, query: &Query, candidates: &[Candidate]) -> Vec<Verdict<Query, Candidate>> {
        let mut verdicts: Vec<Verdict<Query, Candidate>> = candidates.iter()
            .map(|candidate| self.verdict(query, candidate))
            .filter(|verdict| verdict.perfect || verdict.resemblance >= self.floor)
            .collect();

        verdicts.sort_by(|a, b| b.resemblance.partial_cmp(&a.resemblance).unwrap());
        verdicts
    }

    pub fn constrain(&self, query: &Query, candidates: &[Candidate], cap: usize) -> Vec<Verdict<Query, Candidate>> {
        let mut verdicts = self.shortlist(query, candidates);
        verdicts.truncate(cap);
        verdicts
    }
}