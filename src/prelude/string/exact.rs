use {
    crate::{
        assessor::{
            Resembler, Resemblance,
        },
    }
};

#[derive(PartialEq)]
pub struct Exact;

impl Resembler<String, String, ()> for Exact {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            Ok(Resemblance::Perfect)
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(PartialEq)]
pub struct Relaxed;

impl Resembler<String, String, ()> for Relaxed {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query.to_lowercase() == candidate.to_lowercase() {
            Ok(Resemblance::Partial(0.95))
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}