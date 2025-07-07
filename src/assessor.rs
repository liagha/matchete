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

#[derive(Debug)]
pub struct Dimension<Query, Candidate> {
    pub resembler: Box<dyn Resemblance<Query, Candidate>>,
    pub weight: f64,
}

impl<Query, Candidate> Dimension<Query, Candidate> {
    pub fn new<R: Resemblance<Query, Candidate> + 'static>(resembler: R, weight: f64) -> Self {
        Self {
            resembler: Box::new(resembler),
            weight,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assessment {
    pub resemblance: f64,
    pub perfect: bool,
    pub weight: f64,
    pub contribution: f64,
}

#[derive(Debug, Clone)]
pub struct Profile<Query, Candidate> {
    pub query: Query,
    pub candidate: Candidate,
    pub assessments: Vec<Assessment>,
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
        let assessments: Vec<Assessment> = self.dimensions.iter()
            .map(|dimension| {
                let resemblance = dimension.resembler.resemblance(query, candidate);
                let perfect = dimension.resembler.perfect(query, candidate);
                let contribution = resemblance * dimension.weight;

                Assessment {
                    resemblance,
                    perfect,
                    weight: dimension.weight,
                    contribution,
                }
            })
            .collect();

        let total_contribution: f64 = assessments.iter().map(|a| a.contribution).sum();
        let total_weight: f64 = assessments.iter().map(|a| a.weight).sum();
        let total_resemblance = if total_weight > 0.0 { total_contribution / total_weight } else { 0.0 };
        let is_perfect = assessments.iter().any(|a| a.perfect);

        Profile {
            query: query.clone(),
            candidate: candidate.clone(),
            assessments,
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