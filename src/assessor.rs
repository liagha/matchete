use {
    std::sync::Arc,
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
pub struct Dimension<Query, Candidate> {
    pub resembler: Arc<dyn Resemblance<Query, Candidate>>,
    pub weight: f64,
    pub resemblance: f64,
    pub perfect: bool,
    pub contribution: f64,
}

impl<Query, Candidate> Dimension<Query, Candidate> {
    pub fn new<R: Resemblance<Query, Candidate> + 'static>(resembler: R, weight: f64) -> Self {
        Self {
            resembler: Arc::new(resembler),
            weight,
            resemblance: 0.0,
            perfect: false,
            contribution: 0.0,
        }
    }

    pub fn assess(&mut self, query: &Query, candidate: &Candidate) {
        self.resemblance = self.resembler.resemblance(query, candidate);
        self.perfect = self.resembler.perfect(query, candidate);
        self.contribution = self.resemblance * self.weight;
    }
}

#[derive(Debug, Clone)]
pub struct Profile<Query, Candidate> {
    pub query: Query,
    pub candidate: Candidate,
    pub dimensions: Vec<Dimension<Query, Candidate>>,
    pub resemblance: f64,
    pub perfect: bool,
}

impl<Query, Candidate> Profile<Query, Candidate> {
    pub fn viable(&self, floor: f64) -> bool {
        self.perfect || self.resemblance >= floor
    }
}

pub struct Assessor<Query, Candidate> {
    pub dimensions: Vec<Dimension<Query, Candidate>>,
    pub floor: f64,
}

impl<Query, Candidate> Assessor<Query, Candidate> {
    pub fn new() -> Self {
        Self {
            dimensions: Vec::new(),
            floor: 0.4,
        }
    }

    pub fn floor(mut self, floor: f64) -> Self {
        self.floor = floor;
        self
    }

    pub fn dimension<R: Resemblance<Query, Candidate> + 'static>(
        mut self,
        resembler: R,
        weight: f64,
    ) -> Self {
        self.dimensions.push(Dimension::new(resembler, weight));
        self
    }
}

impl<Query, Candidate> Assessor<Query, Candidate>
where
    Query: Clone + Debug,
    Candidate: Clone + Debug,
{
    pub fn profile(&self, query: &Query, candidate: &Candidate) -> Profile<Query, Candidate> {
        let mut dimensions = self.dimensions.clone();

        for dimension in &mut dimensions {
            dimension.assess(query, candidate);
        }

        let total_contribution: f64 = dimensions.iter().map(|d| d.contribution).sum();
        let total_weight: f64 = dimensions.iter().map(|d| d.weight).sum();
        let total_resemblance = if total_weight > 0.0 { total_contribution / total_weight } else { 0.0 };
        let is_perfect = dimensions.iter().all(|d| d.perfect);

        Profile {
            query: query.clone(),
            candidate: candidate.clone(),
            dimensions,
            resemblance: total_resemblance,
            perfect: is_perfect,
        }
    }

    pub fn resemblance(&self, query: &Query, candidate: &Candidate) -> f64 {
        self.profile(query, candidate).resemblance
    }

    pub fn perfect(&self, query: &Query, candidate: &Candidate) -> bool {
        self.profile(query, candidate).perfect
    }

    pub fn viable(&self, query: &Query, candidate: &Candidate) -> bool {
        self.profile(query, candidate).viable(self.floor)
    }

    pub fn champion(&self, query: &Query, candidates: &[Candidate]) -> Option<Profile<Query, Candidate>> {
        candidates.iter()
            .map(|candidate| self.profile(query, candidate))
            .filter(|profile| profile.viable(self.floor))
            .max_by(|a, b| a.resemblance.partial_cmp(&b.resemblance).unwrap())
    }

    pub fn shortlist(&self, query: &Query, candidates: &[Candidate]) -> Vec<Profile<Query, Candidate>> {
        let mut profiles: Vec<Profile<Query, Candidate>> = candidates.iter()
            .map(|candidate| self.profile(query, candidate))
            .filter(|profile| profile.viable(self.floor))
            .collect();

        profiles.sort_by(|a, b| b.resemblance.partial_cmp(&a.resemblance).unwrap());
        profiles
    }

    pub fn constrain(&self, query: &Query, candidates: &[Candidate], cap: usize) -> Vec<Profile<Query, Candidate>> {
        let mut profiles = self.shortlist(query, candidates);
        profiles.truncate(cap);
        profiles
    }
}