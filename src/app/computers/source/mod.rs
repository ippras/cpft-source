use crate::app::{
    MAX_TEMPERATURE,
    panes::source::settings::{Filter, Order, Settings, SortBy},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::{
    polars::expr::{FattyAcidExpr, fatty_acid::kind::FattyAcidExprExt},
    prelude::*,
};
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Source computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Source computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<DataFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        lazy_frame = lazy_frame
            .with_columns([
                // Retention time mean
                col("RetentionTime")
                    .list()
                    .mean()
                    .alias("RetentionTimeMean"),
                // Retention time standard deviation
                col("RetentionTime")
                    .list()
                    .std(key.settings.ddof)
                    .alias("RetentionTimeStandardDeviation"),
            ])
            .with_columns([
                // Relative retention time
                relative_time(key.settings)
                    .over(["Mode"])
                    .alias("RelativeRetentionTime"),
                // Delta retention time
                col("FattyAcid")
                    .fa()
                    .delta(col("RetentionTimeMean"))
                    .over(["Mode"])
                    .alias("DeltaRetentionTime"),
                // Temperature
                (col("Mode").struct_().field_by_name("OnsetTemperature")
                    + col("RetentionTimeMean")
                        * col("Mode").struct_().field_by_name("TemperatureStep"))
                .clip_max(lit(MAX_TEMPERATURE))
                .alias("Temperature"),
                // FCL
                col("FattyAcid")
                    .fa()
                    .fcl(
                        col("RetentionTimeMean"),
                        ChainLengthOptions::new().logarithmic(key.settings.logarithmic),
                    )
                    .over(["Mode"])
                    .alias("FCL"),
                // ECL
                col("FattyAcid")
                    .fa()
                    .ecl(
                        col("RetentionTimeMean"),
                        ChainLengthOptions::new().logarithmic(key.settings.logarithmic),
                    )
                    .over(["Mode"])
                    .alias("EquivalentChainLength"),
                // ECN
                col("FattyAcid").fa().ecn().alias("ECN"),
            ])
            .with_columns([
                // Slope
                col("FattyAcid")
                    .fa()
                    .slope(col("EquivalentChainLength"), col("RetentionTimeMean"))
                    .over(["Mode"])
                    .alias("Slope"),
            ])
            .select([
                col("Mode"),
                col("FattyAcid"),
                // Retention time
                as_struct(vec![
                    as_struct(vec![
                        col("RetentionTimeMean").alias("Mean"),
                        col("RetentionTimeStandardDeviation").alias("StandardDeviation"),
                        col("RetentionTime").alias("Values"),
                    ])
                    .alias("Absolute"),
                    col("RelativeRetentionTime").alias("Relative"),
                    col("DeltaRetentionTime").alias("Delta"),
                ])
                .alias("RetentionTime"),
                // DeadTime
                col("DeadTime"),
                // Temperature
                col("Temperature"),
                // Chain length
                as_struct(vec![col("EquivalentChainLength"), col("FCL"), col("ECN")])
                    .alias("ChainLength"),
                // Mass
                as_struct(vec![
                    col("FattyAcid").fa().rco().mass(None).alias("RCO"),
                    col("FattyAcid").fa().rcoo().mass(None).alias("RCOO"),
                    col("FattyAcid").fa().rcooh().mass(None).alias("RCOOH"),
                    col("FattyAcid").fa().rcooch3().mass(None).alias("RCOOCH3"),
                ])
                .alias("Mass"),
                // Derivative
                as_struct(vec![
                    col("Slope"),
                    col("Slope").arctan().degrees().alias("Angle"),
                ])
                .alias("Derivative"),
            ]);
        // Filter
        if let Some(predicate) = filter(&key.settings.filter) {
            lazy_frame = lazy_frame.filter(predicate);
        }
        // Interpolate
        // Sort
        let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
        if key.settings.order == Order::Descending {
            sort_options = sort_options.with_order_descending(true);
        };
        lazy_frame = match key.settings.sort {
            SortBy::FattyAcid => lazy_frame.sort_by_fatty_acids(sort_options),
            SortBy::Time => lazy_frame.sort_by_time(sort_options),
        };
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        self.try_compute(key).expect("compute source")
    }
}

/// Source key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.kind.hash(state);
        self.settings.ddof.hash(state);
        self.settings.logarithmic.hash(state);
        self.settings.relative.hash(state);
        self.settings.filter.hash(state);
        self.settings.sort.hash(state);
        self.settings.order.hash(state);
    }
}

pub(super) fn filter(filter: &Filter) -> Option<Expr> {
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
                    .and(col("FattyAcid").fa().equal(fatty_acid).not()),
            );
        }
    }
    expr
}

/// Extension methods for [`LazyFrame`]
trait LazyFrameExt {
    fn sort_by_fatty_acids(self, sort_options: SortMultipleOptions) -> LazyFrame;

    fn sort_by_time(self, sort_options: SortMultipleOptions) -> LazyFrame;
}

impl LazyFrameExt for LazyFrame {
    fn sort_by_fatty_acids(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort_by_exprs([col("Mode"), col("FattyAcid")], sort_options)
    }

    fn sort_by_time(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort(["Mode"], sort_options.clone()).select([all()
            .sort_by(
                &[
                    col("ChainLength")
                        .struct_()
                        .field_by_name("EquivalentChainLength"),
                    col("RetentionTime")
                        .struct_()
                        .field_by_name("Absolute")
                        .struct_()
                        .field_by_name("Mean"),
                ],
                sort_options,
            )
            .over([col("Mode")])])
    }
}

fn relative_time(settings: &Settings) -> Expr {
    match &settings.relative {
        Some(relative) => {
            col("RetentionTimeMean")
                / col("RetentionTimeMean")
                    .filter(col("FattyAcid").fa().equal(relative))
                    .first()
        }
        None => lit(f64::NAN),
    }
}

/// Saturated
pub trait Saturated {
    /// Delta
    fn delta(self, expr: Expr) -> Expr;

    /// Slope
    fn slope(self, dividend: Expr, divisor: Expr) -> Expr;

    /// Backward saturated
    fn backward(self, expr: Expr) -> Expr;

    /// Forward saturated
    fn forward(self, expr: Expr) -> Expr;
}

impl Saturated for FattyAcidExpr {
    fn delta(self, expr: Expr) -> Expr {
        self.clone().backward(expr.clone()) - self.clone().forward(expr)
    }

    fn slope(self, dividend: Expr, divisor: Expr) -> Expr {
        self.clone().delta(dividend) / self.clone().delta(divisor)
    }

    fn backward(self, expr: Expr) -> Expr {
        self.clone().saturated_or_null(expr).backward_fill(None)
    }

    fn forward(self, expr: Expr) -> Expr {
        self.clone().saturated_or_null(expr).forward_fill(None)
    }
}

pub(crate) mod plot;
