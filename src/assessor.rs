use {
    std::sync::{Arc, Mutex},
    core::fmt::Debug,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Resemblance {
    Perfect,
    Partial(f64),
    Disparity,
}

impl From<f64> for Resemblance {
    fn from(f: f64) -> Self {
        if f == 0.0 {
            Resemblance::Disparity
        } else if f == 1.0 {
            Resemblance::Perfect
        } else {
            Resemblance::Partial(f)
        }
    }
}

impl From<Resemblance> for f64 {
    fn from(r: Resemblance) -> Self {
        match r {
            Resemblance::Disparity => 0.0,
            Resemblance::Perfect => 1.0,
            Resemblance::Partial(f) => f,
        }
    }
}

impl Resemblance {
    pub fn to_f64(&self) -> f64 {
        match self {
            Resemblance::Disparity => 0.0,
            Resemblance::Perfect => 1.0,
            Resemblance::Partial(f) => *f,
        }
    }
}

pub trait Resembler<Query, Candidate, Error>: Debug {
    fn resemblance(&mut self, query: &Query, candidate: &Candidate) -> Result<Resemblance, Error>;
}

#[derive(Clone, Debug)]
pub struct Dimension<Query, Candidate, Error> {
    pub resembler: Arc<Mutex<dyn Resembler<Query, Candidate, Error>>>,
    pub weight: f64,
    pub resemblance: Resemblance,
    pub contribution: f64,
    pub error: Option<Error>,
}

impl<Query, Candidate, Error> Dimension<Query, Candidate, Error> {
    pub fn new<R: Resembler<Query, Candidate, Error> + 'static>(resembler: R, weight: f64) -> Self {
        Self {
            resembler: Arc::new(Mutex::new(resembler)),
            weight,
            resemblance: Resemblance::Disparity,
            contribution: 0.0,
            error: None,
        }
    }

    pub fn assess(&mut self, query: &Query, candidate: &Candidate) {
        match self.resembler.lock().unwrap().resemblance(query, candidate) {
            Ok(resemblance) => {
                self.resemblance = resemblance;
                self.contribution = self.resemblance.to_f64() * self.weight;
                self.error = None;
            }
            Err(error) => {
                self.resemblance = Resemblance::Disparity;
                self.contribution = 0.0;
                self.error = Some(error);
            }
        }
    }
}

#[derive(Debug)]
pub struct Assessor<Query, Candidate, Error> {
    pub dimensions: Vec<Dimension<Query, Candidate, Error>>,
    pub floor: f64,
    pub errors: Vec<Error>,
}

impl<Query, Candidate, Error> Assessor<Query, Candidate, Error>
where
    Query: Clone + Debug + 'static,
    Candidate: Clone + Debug + 'static,
    Error: Clone + Debug + 'static,
{
    pub fn new() -> Self {
        Self {
            dimensions: Vec::new(),
            floor: 0.4,
            errors: Vec::new(),
        }
    }

    pub fn floor(mut self, floor: f64) -> Self {
        self.floor = floor;
        self
    }

    pub fn dimension<R: Resembler<Query, Candidate, Error> + 'static>(
        mut self,
        resembler: R,
        weight: f64,
    ) -> Self {
        self.dimensions.push(Dimension::new(resembler, weight));
        self
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &[Error] {
        &self.errors
    }
}

impl<Query, Candidate, Error> Resembler<Query, Candidate, Error> for Assessor<Query, Candidate, Error>
where
    Query: Clone + Debug,
    Candidate: Clone + Debug,
    Error: Clone + Debug,
{
    fn resemblance(&mut self, query: &Query, candidate: &Candidate) -> Result<Resemblance, Error> {
        for dimension in &mut self.dimensions {
            dimension.assess(query, candidate);

            if let Some(ref error) = dimension.error {
                return Err(error.clone());
            }
        }

        let total_contribution: f64 = self.dimensions.iter().map(|d| d.contribution).sum();
        let total_weight: f64 = self.dimensions.iter().map(|d| d.weight).sum();
        let value = if total_weight > 0.0 { total_contribution / total_weight } else { 0.0 };

        let result = if value >= 1.0 {
            Resemblance::Perfect
        } else if value > 0.0 {
            Resemblance::Partial(value)
        } else {
            Resemblance::Disparity
        };

        Ok(result)
    }
}

impl<Query, Candidate, Error> Assessor<Query, Candidate, Error>
where
    Query: Clone + Debug,
    Candidate: Clone + Debug,
    Error: Clone,
{
    fn assess_candidate(&mut self, query: &Query, candidate: &Candidate) -> (Resemblance, bool) {
        self.errors.clear();

        // Now directly mutating the dimensions on the assessor
        for dimension in &mut self.dimensions {
            dimension.assess(query, candidate);
            if let Some(ref error) = dimension.error {
                self.errors.push(error.clone());
            }
        }

        let total_contribution: f64 = self.dimensions.iter().map(|d| d.contribution).sum();
        let total_weight: f64 = self.dimensions.iter().map(|d| d.weight).sum();
        let total_resemblance = if total_weight > 0.0 { total_contribution / total_weight } else { 0.0 };

        let resemblance = total_resemblance.into();
        let viable = total_resemblance >= self.floor;

        (resemblance, viable)
    }

    pub fn dominant(&self) -> Option<&Dimension<Query, Candidate, Error>> {
        self.dimensions.iter().max_by(|a, b| a.contribution.partial_cmp(&b.contribution).unwrap())
    }

    pub fn resemblance_value(&mut self, query: &Query, candidate: &Candidate) -> Resemblance {
        self.assess_candidate(query, candidate).0
    }

    pub fn viable(&mut self, query: &Query, candidate: &Candidate) -> bool {
        self.assess_candidate(query, candidate).1
    }

    pub fn champion(&mut self, query: &Query, candidates: &[Candidate]) -> Option<Candidate> {
        let mut best_candidate = None;
        let mut best_resemblance = -1.0;

        for candidate in candidates {
            let (resemblance, viable) = self.assess_candidate(query, candidate);
            let resemblance_val = resemblance.to_f64();

            if viable && resemblance_val > best_resemblance {
                best_resemblance = resemblance_val;
                best_candidate = Some(candidate.clone());
            }
        }

        best_candidate
    }

    pub fn shortlist(&mut self, query: &Query, candidates: &[Candidate]) -> Vec<Candidate> {
        let mut viable_candidates: Vec<(Candidate, f64)> = Vec::new();

        for candidate in candidates {
            let (resemblance, viable) = self.assess_candidate(query, candidate);
            if viable {
                viable_candidates.push((candidate.clone(), resemblance.to_f64()));
            }
        }

        viable_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        viable_candidates.into_iter().map(|(candidate, _)| candidate).collect()
    }

    pub fn constrain(&mut self, query: &Query, candidates: &[Candidate], cap: usize) -> Vec<Candidate> {
        let mut shortlisted = self.shortlist(query, candidates);
        shortlisted.truncate(cap);
        shortlisted
    }
}