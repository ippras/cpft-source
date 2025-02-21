use crate::app::panes::{
    distance::settings::{Aggregation, Settings, Sort, SortBy},
    source::settings::{Filter, Order},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Distance filtered computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Distance filtered computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<DataFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        // Filter
        if let Some(predicate) = filter(&key.settings.filter) {
            lazy_frame = lazy_frame.filter(predicate);
        }
        // Sort
        let (by_exprs, sort_options) = sort(key.settings.sort);
        lazy_frame = lazy_frame.sort_by_exprs(by_exprs, sort_options);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        self.try_compute(key).expect("compute distance filtered")
    }
}

fn sort(sort: Sort) -> (Vec<Expr>, SortMultipleOptions) {
    let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
    if sort.order == Order::Descending {
        sort_options = sort_options.with_order_descending(true);
    };
    let sort_by = match sort.by {
        SortBy::Key => vec![
            col("Mode"),
            col("FattyAcid").struct_().field_by_name("From"),
            col("FattyAcid").struct_().field_by_name("To"),
        ],
        SortBy::Value => vec![col("Alpha").aggregate(sort.aggregation)],
    };
    (sort_by, sort_options)
}

fn filter(filter: &Filter) -> Option<Expr> {
    let mut expr = None;
    if !filter.onset_temperatures.is_empty() {
        for &onset_temperature in &filter.onset_temperatures {
            expr = Some(
                expr.unwrap_or(lit(true)).and(
                    col("Mode")
                        .struct_()
                        .field_by_name("OnsetTemperature")
                        .neq(onset_temperature),
                ),
            );
        }
    }
    if !filter.temperature_steps.is_empty() {
        for &temperature_step in &filter.temperature_steps {
            expr = Some(
                expr.unwrap_or(lit(true)).and(
                    col("Mode")
                        .struct_()
                        .field_by_name("TemperatureStep")
                        .neq(temperature_step),
                ),
            );
        }
    }
    if !filter.fatty_acids.is_empty() {
        for fatty_acid in &filter.fatty_acids {
            expr = Some(
                expr.unwrap_or(lit(true))
                    .and(
                        col("FattyAcid")
                            .struct_()
                            .field_by_name("From")
                            .fa()
                            .equal(fatty_acid)
                            .not(),
                    )
                    .and(
                        col("FattyAcid")
                            .struct_()
                            .field_by_name("To")
                            .fa()
                            .equal(fatty_acid)
                            .not(),
                    ),
            );
        }
    }
    expr
}

/// Extension methods for [`Expr`]
trait ExprExt {
    fn aggregate(self, aggregation: Aggregation) -> Expr;
}

impl ExprExt for Expr {
    fn aggregate(self, aggregation: Aggregation) -> Expr {
        match aggregation {
            Aggregation::Maximum => self.abs().max(),
            Aggregation::Median => self.abs().median(),
            Aggregation::Minimum => self.abs().min(),
        }
        .over([col("Mode")])
    }
}

/// Distance filtered key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.filter.hash(state);
        self.settings.sort.hash(state);
    }
}
