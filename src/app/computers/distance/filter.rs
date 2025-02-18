use crate::app::panes::{
    distance::settings::{Filter, Settings, SortByDistance},
    source::settings::Order,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Distance filter computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Distance filter computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<DataFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        // Filter
        let predicate = filter(&key.settings.filter);
        lazy_frame = lazy_frame.filter(predicate);
        // Sort
        let (by_exprs, sort_options) = sort(&key.settings.sort, &key.settings.order);
        lazy_frame = lazy_frame.sort_by_exprs(by_exprs, sort_options);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        self.try_compute(key).expect("compute distance filter")
    }
}

fn sort(sort: &SortByDistance, order: &Order) -> ([Expr; 1], SortMultipleOptions) {
    let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
    if *order == Order::Descending {
        sort_options = sort_options.with_order_descending(true);
    };
    (
        [match *sort {
            SortByDistance::RetentionTime => col("RetentionTime")
                .struct_()
                .field_by_name("Distance")
                .abs()
                .median()
                .over([col("Mode")]),
            SortByDistance::Ecl => col("ECL")
                .struct_()
                .field_by_name("Distance")
                .abs()
                .median()
                .over([col("Mode")]),
            SortByDistance::Euclidean => col("Distance").abs().median().over([col("Mode")]),
        }],
        sort_options,
    )
}

fn filter(filter: &Filter) -> Expr {
    let mut predicate = lit(true);
    for fatty_acid in &filter.fatty_acids {
        predicate = predicate
            .and(col("From").fa().equal(fatty_acid).not())
            .and(col("To").fa().equal(fatty_acid).not());
    }
    predicate
}

/// Distance filter key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.hash(state);
    }
}
