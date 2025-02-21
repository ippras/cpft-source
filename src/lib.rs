#![feature(tuple_trait)]

pub use app::App;

mod app;
mod localization;
mod presets;
mod special;
mod utils;

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::save;
    use metadata::{MetaDataFrame, Metadata};
    use polars::prelude::*;
    use std::{fs::File, io::Cursor};

    #[test]
    fn test() -> anyhow::Result<()> {
        let bytes = include_bytes!("presets/agilent/Agilent.ipc");
        let agilent = MetaDataFrame::read(Cursor::new(bytes)).unwrap();
        let bytes = include_bytes!("presets/agilent/DeadTime.ipc");
        let dead_time = MetaDataFrame::read(Cursor::new(bytes)).unwrap();
        println!("agilent: {}", agilent.data);
        let mut data = agilent
            .data
            .lazy()
            .join(
                dead_time.data.lazy(),
                [col("Mode").struct_().field_by_name("OnsetTemperature")],
                [col("OnsetTemperature")],
                JoinArgs::new(JoinType::Left),
            )
            .drop([col("OnsetTemperature")])
            .collect()?;
        // println!("agilent: {}", data.collect()?);

        let frame = MetaDataFrame::new(&agilent.meta, &mut data);
        save("temp.ipc", frame)?;
        Ok(())
    }
}
