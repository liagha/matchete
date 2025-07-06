use core::fmt::Debug;

pub trait Scorer<Query, Item>: Debug {
    fn score(&self, query: &Query, item: &Item) -> f64;

    fn exact(&self, query: &Query, item: &Item) -> bool {
        self.score(query, item) >= 0.9999
    }
}

#[derive(Debug, Clone)]
pub struct Weight {
    pub value: f64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Detail {
    pub score: f64,
    pub weight: Weight,
    pub weighted: f64,
}

impl Detail {
    pub fn new(score: f64, weight: Weight) -> Self {
        let weighted = score * weight.value;
        Self { score, weight, weighted }
    }
}

#[derive(Debug, Clone)]
pub struct Result<Query, Item> {
    pub query: Query,
    pub item: Item,
    pub score: f64,
    pub exact: bool,
    pub details: Vec<Detail>,
}

#[derive(Debug, Clone)]
pub enum Kind {
    Exact,
    Similar,
    None,
}