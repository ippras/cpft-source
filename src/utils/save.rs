use anyhow::Result;
use metadata::{MetaDataFrame, Metadata};
use polars::prelude::DataFrame;
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
pub fn save(name: &str, frame: MetaDataFrame<&Metadata, &mut DataFrame>) -> Result<()> {
    let file = File::create(name)?;
    MetaDataFrame::new(frame.meta.clone(), frame.data).write(file)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn save(name: &str, frame: MetaDataFrame<&Metadata, &mut DataFrame>) -> Result<()> {
    use anyhow::anyhow;
    use egui_ext::download;

    let mut bytes = Vec::new();
    MetaDataFrame::new(frame.meta.clone(), frame.data).write(&mut bytes)?;
    download(name, &bytes).map_err(|error| anyhow!(error))
}
