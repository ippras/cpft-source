use crate::app::{
    MAX_TEMPERATURE,
    panes::source::settings::{Filter, Group, Kind, Order, Settings, SortBy},
};
use egui::{
    emath::{Float, OrderedFloat},
    util::cache::{ComputerMut, FrameCache},
};
use egui_plot::{Line, PlotPoint, PlotPoints};
use lipid::{
    polars::{
        data_frame,
        expr::{FattyAcidExpr, fatty_acid::kind::FattyAcidExprExt},
    },
    prelude::*,
};
use polars::prelude::{array::ArrayNameSpace, *};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

/// Source plot computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Source plot computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        // println!(
        //     "lazy_frame SOURCE: {}",
        //     lazy_frame.clone().collect().unwrap()
        // );
        lazy_frame = lazy_frame.select([
            col("Mode").struct_().field_by_name("OnsetTemperature"),
            col("Mode").struct_().field_by_name("TemperatureStep"),
            col("FattyAcid"),
            col("RetentionTime")
                .struct_()
                .field_by_name("Absolute")
                .struct_()
                .field_by_name("Mean")
                .alias("RetentionTime"),
            col("ChainLength")
                .struct_()
                .field_by_name("EquivalentChainLength")
                .alias("EquivalentChainLength"),
        ]);
        // println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = lazy_frame.select([
            col("OnsetTemperature"),
            col("TemperatureStep"),
            col("FattyAcid"),
            concat_arr(vec![col("RetentionTime"), col("EquivalentChainLength")])?.alias("Points"),
        ]);
        let lazy_frame1 = lazy_frame
            .group_by([col("FattyAcid"), col("OnsetTemperature")])
            .agg([col("TemperatureStep"), col("Points")]);
        let data_frame = lazy_frame1.collect()?;
        let mut value = Value::default();
        for (((fatty_acid, onset_temperature), temperature_steps), points) in
            data_frame["FattyAcid"]
                .fa()
                .into_iter()
                .zip(data_frame["OnsetTemperature"].f64()?.into_no_null_iter())
                .zip(data_frame["TemperatureStep"].list()?.into_no_null_iter())
                .zip(data_frame["Points"].list()?.into_no_null_iter())
        {
            let fatty_acid = fatty_acid.unwrap(); // TODO
            let mut line_points = Vec::new();
            for (temperature_step, points) in temperature_steps
                .f64()?
                .into_no_null_iter()
                .zip(points.array()?.into_no_null_iter())
            {
                let points = points.f64()?;
                let Some(x) = points.get(0) else {
                    continue;
                };
                let Some(y) = points.get(1) else {
                    continue;
                };
                line_points.push(PlotPoint::new(x, y));
                value
                    .points
                    .entry(PointKey::new(x, y))
                    .or_default()
                    .insert(PointValue {
                        onset_temperature,
                        temperature_step,
                    });
            }
            value.lines.temperature_step.push(TemperatureStepLine {
                fatty_acid,
                onset_temperature,
                points: line_points,
            });
            // value.onset_temperatures.push(OnsetTemperaturePoints {
            //     fatty_acid,
            //     onset_temperature,
            //     points: line_points,
            // });
            // let index = Int64Chunked::from_vec(PlSmallStr::EMPTY, vec![1]);
            // let z = points.array()?.array_get(&index, false)?;
        }
        Ok(value)

        // // let lazy_frame = lazy_frame.rank()
        // let lazy_frame = lazy_frame
        //     .clone()
        //     .group_by([col("FattyAcid"), col("OnsetTemperature")])
        //     .agg([
        //         col("TemperatureStep"),
        //         col("RetentionTime"),
        //         col("EquivalentChainLength"),
        //     ])
        //     .group_by([col("FattyAcid")])
        //     .agg([
        //         col("OnsetTemperature"),
        //         col("TemperatureStep"),
        //         col("RetentionTime"),
        //         col("EquivalentChainLength"),
        //     ]);

        // let lazy_frame2 = lazy_frame
        //     .clone()
        //     .group_by([col("FattyAcid"), col("TemperatureStep")])
        //     .agg([
        //         col("OnsetTemperature"),
        //         col("RetentionTime"),
        //         col("EquivalentChainLength"),
        //     ])
        //     .group_by([col("FattyAcid")])
        //     .agg([
        //         col("TemperatureStep"),
        //         col("OnsetTemperature"),
        //         col("RetentionTime"),
        //         col("EquivalentChainLength"),
        //     ])
        //     .sort(["FattyAcid"], Default::default());
        // println!(
        //     "lazy_frame2 by FattyAcid/TemperatureStep/OnsetTemperature: {}",
        //     lazy_frame2.clone().collect().unwrap()
        // );
        // lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key<'_>) -> Value {
        self.try_compute(key).expect("compute plot source")
    }
}

/// Source plot key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.ddof.hash(state);
        self.settings.logarithmic.hash(state);
        self.settings.filter.hash(state);
        self.settings.radius_of_points.hash(state);
    }
}

/// Source plot value
#[derive(Clone, Default)]
pub(crate) struct Value {
    pub(crate) lines: Lines,
    pub(crate) points: HashMap<PointKey, HashSet<PointValue>>,
}

#[derive(Clone, Default)]
pub(crate) struct Lines {
    pub(crate) temperature_step: Vec<TemperatureStepLine>,
    // pub(crate) onset_temperature: Vec<OnsetTemperatureLine>,
}

#[derive(Clone)]
pub(crate) struct TemperatureStepLine {
    pub(crate) fatty_acid: FattyAcid,
    pub(crate) onset_temperature: f64,
    pub(crate) points: Vec<PlotPoint>,
}

// #[derive(Clone)]
// pub(crate) struct OnsetTemperatureLine {
//     pub(crate) fatty_acid: FattyAcid,
//     pub(crate) temperature_step: f64,
//     pub(crate) points: Vec<PlotPoint>,
// }

#[derive(Clone, Copy, Debug)]
pub(crate) struct PointKey {
    pub(crate) retention_time: f64,
    pub(crate) equivalent_chain_length: f64,
}

impl PointKey {
    pub(crate) fn new(retention_time: f64, equivalent_chain_length: f64) -> Self {
        Self {
            retention_time,
            equivalent_chain_length,
        }
    }
}

impl Eq for PointKey {}

impl From<PointKey> for PlotPoint {
    fn from(value: PointKey) -> Self {
        PlotPoint {
            x: value.retention_time,
            y: value.equivalent_chain_length,
        }
    }
}

impl Hash for PointKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.retention_time.ord().hash(state);
        self.equivalent_chain_length.ord().hash(state);
    }
}

impl PartialEq for PointKey {
    fn eq(&self, other: &Self) -> bool {
        self.retention_time.ord() == other.retention_time.ord()
            && self.equivalent_chain_length.ord() == other.equivalent_chain_length.ord()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PointValue {
    pub(crate) onset_temperature: f64,
    pub(crate) temperature_step: f64,
}

impl Eq for PointValue {}

impl Hash for PointValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.ord().hash(state);
        self.temperature_step.ord().hash(state);
    }
}

impl PartialEq for PointValue {
    fn eq(&self, other: &Self) -> bool {
        self.onset_temperature.ord() == other.onset_temperature.ord()
            && self.temperature_step.ord() == other.temperature_step.ord()
    }
}
