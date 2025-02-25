use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use polars_ext::ExprExt;
use std::hash::{Hash, Hasher};

/// Distance computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Distance computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<DataFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        // Join
        lazy_frame = lazy_frame
            .clone()
            .select([
                col("Mode").hash().alias("LeftHash"),
                as_struct(vec![
                    col("FattyAcid"),
                    col("RetentionTime")
                        .struct_()
                        .field_by_name("Absolute")
                        .struct_()
                        .field_by_name("Mean")
                        .name()
                        .keep(),
                    col("ChainLength")
                        .struct_()
                        .field_by_name("EquivalentChainLength"),
                ])
                .alias("From"),
                col("Mode"),
                col("DeadTime"),
            ])
            .with_row_index("LeftIndex", None)
            .join_builder()
            .with(
                lazy_frame
                    .select([
                        col("Mode").hash().alias("RightHash"),
                        as_struct(vec![
                            col("FattyAcid"),
                            col("RetentionTime")
                                .struct_()
                                .field_by_name("Absolute")
                                .struct_()
                                .field_by_name("Mean")
                                .name()
                                .keep(),
                            col("ChainLength")
                                .struct_()
                                .field_by_name("EquivalentChainLength"),
                        ])
                        .alias("To"),
                    ])
                    .with_row_index("RightIndex", None),
            )
            .join_where(vec![
                // Same modes
                col("LeftHash").eq(col("RightHash")),
                // Fatty asids not equals combination
                col("LeftIndex").lt(col("RightIndex")),
            ]);
        // Select
        lazy_frame = lazy_frame
            .select([
                col("Mode"),
                col("DeadTime"),
                as_struct(vec![
                    col("From")
                        .struct_()
                        .field_by_name("FattyAcid")
                        .name()
                        .keep(),
                    col("To").struct_().field_by_name("FattyAcid").name().keep(),
                ])
                .alias("FattyAcid"),
                as_struct(vec![
                    col("From")
                        .struct_()
                        .field_by_name("RetentionTime")
                        .name()
                        .keep(),
                    col("To")
                        .struct_()
                        .field_by_name("RetentionTime")
                        .name()
                        .keep(),
                    (col("To").struct_().field_by_name("RetentionTime")
                        - col("From").struct_().field_by_name("RetentionTime"))
                    .over([col("Mode")])
                    .alias("Delta"),
                ])
                .alias("RetentionTime"),
                as_struct(vec![
                    col("From")
                        .struct_()
                        .field_by_name("EquivalentChainLength")
                        .name()
                        .keep(),
                    col("To")
                        .struct_()
                        .field_by_name("EquivalentChainLength")
                        .name()
                        .keep(),
                    (col("To").struct_().field_by_name("EquivalentChainLength")
                        - col("From").struct_().field_by_name("EquivalentChainLength"))
                    .over([col("Mode")])
                    .alias("Delta"),
                ])
                .alias("EquivalentChainLength"),
                ((col("From").struct_().field_by_name("RetentionTime") - col("DeadTime"))
                    / (col("To").struct_().field_by_name("RetentionTime") - col("DeadTime"))
                        .over([col("Mode")]))
                .alias("Alpha"),
            ])
            .with_column(
                (col("RetentionTime").struct_().field_by_name("Delta").pow(2)
                    + col("EquivalentChainLength")
                        .struct_()
                        .field_by_name("Delta")
                        .pow(2))
                .sqrt()
                .alias("EuclideanDistance"),
            );
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        self.try_compute(key).expect("compute distance")
    }
}

/// Distance key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) hash: u64,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

pub(crate) mod filtered;
pub(crate) mod plot;
