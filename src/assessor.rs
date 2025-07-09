use {
    std::sync::Arc,
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
    fn resemblance(&self, query: &Query, candidate: &Candidate) -> Result<Resemblance, Error>;
}

#[derive(Clone, Debug)]
pub struct Dimension<Query, Candidate, Error> {
    pub resembler: Arc<dyn Resembler<Query, Candidate, Error>>,
    pub weight: f64,
    pub resemblance: Resemblance,
    pub contribution: f64,
    pub error: Option<Error>,
}

impl<Query, Candidate, Error> Dimension<Query, Candidate, Error> {
    pub fn new<R: Resembler<Query, Candidate, Error> + 'static>(resembler: R, weight: f64) -> Self {
        Self {
            resembler: Arc::new(resembler),
            weight,
            resemblance: Resemblance::Disparity,
            contribution: 0.0,
            error: None,
        }
    }

    pub fn assess(&mut self, query: &Query, candidate: &Candidate) {
        match self.resembler.resemblance(query, candidate) {
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
pub struct Blend<Query, Candidate, Error> {
    dimensions: Vec<Dimension<Query, Candidate, Error>>,
    blender: Blender,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Blender {
    Weighted,
    Minimum,
    Maximum,
    Product,
}

impl<Query, Candidate, Error> Blend<Query, Candidate, Error> {
    pub fn weighted(dimensions: Vec<Dimension<Query, Candidate, Error>>) -> Self {
        Self {
            dimensions,
            blender: Blender::Weighted,
        }
    }

    pub fn minimum(dimensions: Vec<Dimension<Query, Candidate, Error>>) -> Self {
        Self {
            dimensions,
            blender: Blender::Minimum,
        }
    }

    pub fn maximum(dimensions: Vec<Dimension<Query, Candidate, Error>>) -> Self {
        Self {
            dimensions,
            blender: Blender::Maximum,
        }
    }

    pub fn product(dimensions: Vec<Dimension<Query, Candidate, Error>>) -> Self {
        Self {
            dimensions,
            blender: Blender::Product,
        }
    }

    fn compute(&self, resemblances: &[f64], weights: &[f64]) -> f64 {
        match self.blender {
            Blender::Weighted => {
                let total_contribution: f64 = resemblances.iter().zip(weights.iter()).map(|(r, w)| r * w).sum();
                let total_weight: f64 = weights.iter().sum();
                if total_weight > 0.0 { total_contribution / total_weight } else { 0.0 }
            }
            Blender::Minimum => resemblances.iter().cloned().fold(f64::INFINITY, f64::min),
            Blender::Maximum => resemblances.iter().cloned().fold(0.0, f64::max),
            Blender::Product => resemblances.iter().product(),
        }
    }
}

impl<Query, Candidate, Error> Resembler<Query, Candidate, Error> for Blend<Query, Candidate, Error>
where
    Query: Clone + Debug,
    Candidate: Clone + Debug,
    Error: Clone + Debug,
{
    fn resemblance(&self, query: &Query, candidate: &Candidate) -> Result<Resemblance, Error> {
        let mut dimensions = self.dimensions.clone();

        for dimension in &mut dimensions {
            dimension.assess(query, candidate);

            if let Some(ref error) = dimension.error {
                return Err(error.clone());
            }
        }

        let resemblances: Vec<f64> = dimensions.iter().map(|d| d.resemblance.to_f64()).collect();
        let weights: Vec<f64> = dimensions.iter().map(|d| d.weight).collect();

        let value = self.compute(&resemblances, &weights);
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

#[derive(Debug)]
pub struct Union<Query, Candidate, Error> {
    resemblers: Vec<Arc<dyn Resembler<Query, Candidate, Error>>>,
    weights: Vec<f64>,
}

impl<Query, Candidate, Error> Union<Query, Candidate, Error> {
    pub fn new() -> Self {
        Self {
            resemblers: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add<R: Resembler<Query, Candidate, Error> + 'static>(mut self, resembler: R, weight: f64) -> Self {
        self.resemblers.push(Arc::new(resembler));
        self.weights.push(weight);
        self
    }
}

impl<Query, Candidate, Error> Resembler<Query, Candidate, Error> for Union<Query, Candidate, Error>
where
    Query: Clone + Debug,
    Candidate: Clone + Debug,
    Error: Clone + Debug,
{
    fn resemblance(&self, query: &Query, candidate: &Candidate) -> Result<Resemblance, Error> {
        let mut total_contribution = 0.0;
        let mut successful_weight = 0.0;

        for (resembler, weight) in self.resemblers.iter().zip(self.weights.iter()) {
            match resembler.resemblance(query, candidate) {
                Ok(resemblance) => {
                    total_contribution += resemblance.to_f64() * weight;
                    successful_weight += weight;
                }
                Err(error) => {
                    // For Union, we could choose to continue with other resemblers
                    // or return the first error. Here we return the first error.
                    return Err(error);
                }
            }
        }

        let value = if successful_weight > 0.0 { total_contribution / successful_weight } else { 0.0 };
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

#[derive(Debug, Clone)]
pub struct Profile<Query, Candidate, Error> {
    pub query: Query,
    pub candidate: Candidate,
    pub dimensions: Vec<Dimension<Query, Candidate, Error>>,
    pub resemblance: Resemblance,
}

impl<Query, Candidate, Error> Profile<Query, Candidate, Error> {
    pub fn viable(&self, floor: f64) -> bool {
        self.resemblance.to_f64() >= floor
    }

    pub fn has_errors(&self) -> bool {
        self.dimensions.iter().any(|d| d.error.is_some())
    }

    pub fn successful(&self) -> bool {
        !self.has_errors()
    }

    pub fn errors(&self) -> Vec<&Error> {
        self.dimensions.iter().filter_map(|d| d.error.as_ref()).collect()
    }
}

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

    pub fn blend(mut self, blend: Blend<Query, Candidate, Error>, weight: f64) -> Self {
        self.dimensions.push(Dimension::new(blend, weight));
        self
    }

    pub fn union(mut self, union: Union<Query, Candidate, Error>, weight: f64) -> Self {
        self.dimensions.push(Dimension::new(union, weight));
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

impl<Query, Candidate, Error> Assessor<Query, Candidate, Error>
where
    Query: Clone + Debug,
    Candidate: Clone + Debug,
    Error: Clone,
{
    pub fn profile(&mut self, query: &Query, candidate: &Candidate) -> Profile<Query, Candidate, Error> {
        let mut dimensions = self.dimensions.clone();

        // Clear previous errors
        self.errors.clear();

        for dimension in &mut dimensions {
            dimension.assess(query, candidate);
            if let Some(ref error) = dimension.error {
                self.errors.push(error.clone());
            }
        }

        let total_contribution: f64 = dimensions.iter().map(|d| d.contribution).sum();
        let total_weight: f64 = dimensions.iter().map(|d| d.weight).sum();
        let total_resemblance = if total_weight > 0.0 { total_contribution / total_weight } else { 0.0 };

        Profile {
            query: query.clone(),
            candidate: candidate.clone(),
            dimensions,
            resemblance: total_resemblance.into(),
        }
    }

    pub fn resemblance(&mut self, query: &Query, candidate: &Candidate) -> Resemblance {
        self.profile(query, candidate).resemblance
    }

    pub fn viable(&mut self, query: &Query, candidate: &Candidate) -> bool {
        self.profile(query, candidate).viable(self.floor)
    }

    pub fn champion(&mut self, query: &Query, candidates: &[Candidate]) -> Option<Profile<Query, Candidate, Error>> {
        let mut best_profile = None;
        let mut best_resemblance = -1.0;

        for candidate in candidates {
            let profile = self.profile(query, candidate);
            let resemblance_val = profile.resemblance.to_f64();

            if profile.viable(self.floor) && resemblance_val > best_resemblance {
                best_resemblance = resemblance_val;
                best_profile = Some(profile);
            }
        }

        best_profile
    }

    pub fn shortlist(&mut self, query: &Query, candidates: &[Candidate]) -> Vec<Profile<Query, Candidate, Error>> {
        let mut profiles: Vec<Profile<Query, Candidate, Error>> = Vec::new();

        for candidate in candidates {
            let profile = self.profile(query, candidate);
            if profile.viable(self.floor) {
                profiles.push(profile);
            }
        }

        profiles.sort_by(|a, b| b.resemblance.to_f64().partial_cmp(&a.resemblance.to_f64()).unwrap());
        profiles
    }

    pub fn constrain(&mut self, query: &Query, candidates: &[Candidate], cap: usize) -> Vec<Profile<Query, Candidate, Error>> {
        let mut profiles = self.shortlist(query, candidates);
        profiles.truncate(cap);
        profiles
    }

    pub fn all_profiles(&mut self, query: &Query, candidates: &[Candidate]) -> Vec<Profile<Query, Candidate, Error>> {
        candidates.iter()
            .map(|candidate| self.profile(query, candidate))
            .collect()
    }

    pub fn error_profiles(&mut self, query: &Query, candidates: &[Candidate]) -> Vec<Profile<Query, Candidate, Error>> {
        candidates.iter()
            .map(|candidate| self.profile(query, candidate))
            .filter(|profile| profile.has_errors())
            .collect()
    }
}