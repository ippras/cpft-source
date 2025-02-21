use polars::prelude::*;
use std::{error::Error, fmt::Arguments, marker::Tuple};

pub fn unwrap_f<T, U: Tuple>(
    f: impl FnOnce(U) -> Result<T, Box<dyn Error>>,
) -> impl FnOnce(U) -> T {
    |u| f(u).unwrap()
}
