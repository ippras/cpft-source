use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Distance unique computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Distance unique computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<DataFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        lazy_frame = lazy_frame.select([col("FattyAcid").unique().sort(Default::default())]);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        self.try_compute(key).expect("compute distance unique")
    }
}

/// Distance unique key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in self.data_frame["FattyAcid"].as_materialized_series().iter() {
            value.hash(state);
        }
    }
}
