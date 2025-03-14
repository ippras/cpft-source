use anyhow::Result;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) data_frame: DataFrame,
}

impl Data {
    pub(crate) fn stack(&mut self, data_frame: &DataFrame) -> Result<()> {
        // If many vstack operations are done, it is recommended to call DataFrame::align_chunks_par
        self.data_frame.vstack_mut(data_frame)?.align_chunks_par();
        Ok(())
    }

    pub(crate) fn join(&mut self, data_frame: DataFrame) -> Result<()> {
        self.data_frame = self
            .data_frame
            .clone()
            .lazy()
            .unnest(["FA"])
            .join(
                data_frame.lazy().unnest(["FA"]),
                [
                    col("Carbons"),
                    col("Indices"),
                    col("Bounds"),
                    col("Label"),
                    col("OnsetTemperature"),
                    col("TemperatureStep"),
                    col("Time"),
                ],
                [
                    col("Carbons"),
                    col("Indices"),
                    col("Bounds"),
                    col("Label"),
                    col("OnsetTemperature"),
                    col("TemperatureStep"),
                    col("Time"),
                ],
                JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
            )
            .select([
                as_struct(vec![
                    col("Carbons"),
                    col("Indices"),
                    col("Bounds"),
                    col("Label"),
                ])
                .alias("FA"),
                col("OnsetTemperature"),
                col("TemperatureStep"),
                col("Time"),
            ])
            .collect()?;
        // println!("self.data_frame: {}", self.data_frame);
        Ok(())
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.data_frame, f)
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            data_frame: DataFrame::empty_with_schema(&Schema::from_iter([
                Field::new(
                    "FA".into(),
                    DataType::Struct(vec![
                        Field::new("Carbons".into(), DataType::UInt8),
                        Field::new("Indices".into(), DataType::List(Box::new(DataType::UInt8))),
                        Field::new("Bounds".into(), DataType::List(Box::new(DataType::Int8))),
                        Field::new("Label".into(), DataType::String),
                    ]),
                ),
                Field::new("OnsetTemperature".into(), DataType::Float64),
                Field::new("TemperatureStep".into(), DataType::Float64),
                Field::new("Time".into(), DataType::List(Box::new(DataType::Float64))),
            ])),
        }
    }
}
