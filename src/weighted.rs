use core::{fmt::Debug, marker::PhantomData};
use std::fmt::Formatter;
use crate::{Scorer, Weight, Detail};

pub struct Weighted<Query, Item, S> {
    scorer: S,
    weight: Weight,
    phantom: PhantomData<(Query, Item)>,
}

impl<Query, Item, S> Weighted<Query, Item, S> {
    pub fn new<N: Into<String>>(scorer: S, weight: f64, name: N) -> Self {
        Self {
            scorer,
            weight: Weight {
                value: weight,
                name: name.into(),
            },
            phantom: PhantomData,
        }
    }

    pub fn detail(&self, query: &Query, item: &Item) -> Detail
    where
        S: Scorer<Query, Item>
    {
        let score = self.scorer.score(query, item);
        Detail::new(score, self.weight.clone())
    }
}

impl<Query, Item, S> Debug for Weighted<Query, Item, S>
where
    S: Scorer<Query, Item>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Weighted({:?} | {:?})", self.weight, self.scorer)
    }
}

impl<Query, Item, S> Scorer<Query, Item> for Weighted<Query, Item, S>
where
    S: Scorer<Query, Item>,
{
    fn score(&self, query: &Query, item: &Item) -> f64 {
        self.scorer.score(query, item) * self.weight.value
    }

    fn exact(&self, query: &Query, item: &Item) -> bool {
        self.scorer.exact(query, item)
    }
}