use core::{fmt::Debug, marker::PhantomData};
use crate::Scorer;

pub struct Custom<Query, Item, F> {
    name: String,
    func: F,
    phantom: PhantomData<(Query, Item)>,
}

impl<Query, Item, F> Custom<Query, Item, F>
where
    F: Fn(&Query, &Item) -> f64,
{
    pub fn new<N: Into<String>>(name: N, func: F) -> Self {
        Self {
            name: name.into(),
            func,
            phantom: PhantomData,
        }
    }
}

impl<Query, Item, F> Debug for Custom<Query, Item, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Custom({})", self.name)
    }
}

impl<Query, Item, F> Scorer<Query, Item> for Custom<Query, Item, F>
where
    F: Fn(&Query, &Item) -> f64,
{
    fn score(&self, query: &Query, item: &Item) -> f64 {
        (self.func)(query, item)
    }
}