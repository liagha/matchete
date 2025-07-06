use crate::Scorer;

#[derive(Debug)]
pub struct NumericProximityScorer {
    pub normalization_factor: f64,
}

impl Default for NumericProximityScorer {
    fn default() -> Self {
        NumericProximityScorer {
            normalization_factor: 10.0,
        }
    }
}

impl NumericProximityScorer {
    pub fn new(normalization_factor: f64) -> Self {
        NumericProximityScorer {
            normalization_factor,
        }
    }
}

impl<T, U> Scorer<T, U> for NumericProximityScorer
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn score(&self, query: &T, item: &U) -> f64 {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        let difference = (query_val - item_val).abs();
        let normalized_diff = difference / self.normalization_factor;

        (-normalized_diff).exp()
    }

    fn exact(&self, query: &T, item: &U) -> bool {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        query_val == item_val
    }
}

#[derive(Debug)]
pub struct RangeProximityScorer {
    pub min_value: f64,
    pub max_value: f64,
}

impl Default for RangeProximityScorer {
    fn default() -> Self {
        RangeProximityScorer {
            min_value: 0.0,
            max_value: 100.0,
        }
    }
}

impl RangeProximityScorer {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        RangeProximityScorer {
            min_value,
            max_value,
        }
    }
}

impl<T, U> Scorer<T, U> for RangeProximityScorer
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn score(&self, query: &T, item: &U) -> f64 {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        if query_val == item_val {
            return 1.0;
        }

        let range = self.max_value - self.min_value;
        if range <= 0.0 {
            return if query_val == item_val { 1.0 } else { 0.0 };
        }

        let max_distance = range;
        let actual_distance = (query_val - item_val).abs();

        1.0 - (actual_distance / max_distance).min(1.0)
    }

    fn exact(&self, query: &T, item: &U) -> bool {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        query_val == item_val
    }
}

#[derive(Debug)]
pub struct PercentageProximityScorer;

impl<T, U> Scorer<T, U> for PercentageProximityScorer
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn score(&self, query: &T, item: &U) -> f64 {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        if query_val == item_val {
            return 1.0;
        }

        if query_val == 0.0 || item_val == 0.0 {
            return 0.0;
        }

        let larger = query_val.max(item_val);
        let smaller = query_val.min(item_val);

        smaller / larger
    }

    fn exact(&self, query: &T, item: &U) -> bool {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        query_val == item_val
    }
}

#[derive(Debug)]
pub struct ExponentialDecayScorer {
    pub decay_factor: f64,
}

impl Default for ExponentialDecayScorer {
    fn default() -> Self {
        ExponentialDecayScorer {
            decay_factor: 0.1,
        }
    }
}

impl ExponentialDecayScorer {
    pub fn new(decay_factor: f64) -> Self {
        ExponentialDecayScorer {
            decay_factor,
        }
    }
}

impl<T, U> Scorer<T, U> for ExponentialDecayScorer
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn score(&self, query: &T, item: &U) -> f64 {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        let difference = (query_val - item_val).abs();

        (-self.decay_factor * difference).exp()
    }

    fn exact(&self, query: &T, item: &U) -> bool {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        query_val == item_val
    }
}

#[derive(Debug)]
pub struct ThresholdProximityScorer {
    pub threshold: f64,
}

impl Default for ThresholdProximityScorer {
    fn default() -> Self {
        ThresholdProximityScorer {
            threshold: 5.0,
        }
    }
}

impl ThresholdProximityScorer {
    pub fn new(threshold: f64) -> Self {
        ThresholdProximityScorer {
            threshold,
        }
    }
}

impl<T, U> Scorer<T, U> for ThresholdProximityScorer
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn score(&self, query: &T, item: &U) -> f64 {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        let difference = (query_val - item_val).abs();

        if difference <= self.threshold {
            1.0 - (difference / self.threshold)
        } else {
            0.0
        }
    }

    fn exact(&self, query: &T, item: &U) -> bool {
        let query_val: f64 = (*query).into();
        let item_val: f64 = (*item).into();

        query_val == item_val
    }
}