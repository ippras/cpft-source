use super::Settings;
use crate::app::computers::{SourcePlotValue, plot::IndexKey};
use egui::Ui;
use egui_ext::color;
use egui_l20n::UiExt;
use egui_plot::{Legend, Line, LineStyle, MarkerShape, Plot, PlotPoint, PlotPoints, Points};
use itertools::Itertools;
use lipid::fatty_acid::{
    FattyAcidExt as _,
    display::{COMMON, DisplayWithOptions as _},
};
use polars::prelude::*;
use std::fmt::Write;
use tracing::error;

/// Plot view
#[derive(Clone)]
pub(crate) struct PlotView<'a> {
    pub(crate) data: SourcePlotValue,
    pub(crate) settings: &'a Settings,
}

impl<'a> PlotView<'a> {
    pub(crate) fn new(data: SourcePlotValue, settings: &'a Settings) -> Self {
        Self { data, settings }
    }
}

impl PlotView<'_> {
    pub(super) fn show(self, ui: &mut Ui) {
        if let Err(error) = self.try_show(ui) {
            error!(%error);
        }
    }

    fn try_show(self, ui: &mut Ui) -> PolarsResult<()> {
        let mut plot = Plot::new("plot")
            // .allow_drag(context.settings.visualization.drag)
            // .allow_scroll(context.settings.visualization.scroll)
            ;
        if self.settings.legend {
            plot = plot.legend(Legend::default().follow_insertion_order(true));
        }
        // let scale = plot.transform.dvalue_dpos();
        // let x_decimals = ((-scale[0].abs().log10()).ceil().at_least(0.0) as usize).clamp(1, 6);
        // let y_decimals = ((-scale[1].abs().log10()).ceil().at_least(0.0) as usize).clamp(1, 6);
        let onset_temperature = ui.localize("onset-temperature");
        let temperature_step = ui.localize("temperature-step");
        let retention_time = ui.localize("retention-time");
        let equivalent_chain_length = ui.localize("equivalent-chain-length");
        let points = self.data.index.clone();
        plot = plot
            .x_axis_label(&retention_time)
            .y_axis_label(&equivalent_chain_length)
            .label_formatter(move |name, PlotPoint { x, y }| {
                let mut label = String::new();
                if !name.is_empty() {
                    writeln!(&mut label, "{name}").ok();
                }
                if let Some(values) = points.get(&IndexKey(PlotPoint::new(*x, *y))) {
                    writeln!(
                        &mut label,
                        "{onset_temperature} = {}",
                        values
                            .iter()
                            .map(|value| value.onset_temperature)
                            .format(","),
                    )
                    .ok();
                    writeln!(
                        &mut label,
                        "{temperature_step} = {}",
                        values
                            .iter()
                            .map(|value| value.temperature_step)
                            .format(","),
                    )
                    .ok();
                }
                let precision = self.settings.precision;
                writeln!(&mut label, "{retention_time} = {x:.precision$}").ok();
                writeln!(&mut label, "{equivalent_chain_length} = {y:.precision$}").ok();
                label
            });
        plot.show(ui, |ui| -> PolarsResult<()> {
            for data in &self.data.lines.temperature_step {
                // Line
                let name = format!("{:#}", (&data.fatty_acid).display(COMMON));
                let mut line = Line::new(PlotPoints::Borrowed(&data.points))
                    .name(&name)
                    .color(color(data.onset_temperature as _));
                if data.fatty_acid.is_unsaturated() {
                    line = line.style(LineStyle::Dashed { length: 16.0 });
                }
                ui.line(line);
                // Points
                let mut points = Points::new(PlotPoints::Borrowed(&data.points))
                    .name(name)
                    .color(color(data.onset_temperature as _))
                    .radius(self.settings.radius_of_points);
                if data.fatty_acid.is_saturated() {
                    points = points.shape(MarkerShape::Square);
                }
                ui.points(points);
            }
            // let mut points = Points::new(PlotPoints::Owned(self.data.points))
            //     .name(format!(
            //         "{:#} {temperature_step}",
            //         (&fatty_acid).display(COMMON)
            //     ))
            //     .color(color(onset_temperature as _))
            //     // .name(onset_temperature)
            //     // .color(color(onset_temperature as _))
            //     .radius(3.0);
            // if fatty_acid.unsaturation() == 0 {
            //     points = points.shape(MarkerShape::Square);
            // }

            // for (fatty_acid, onset_temperature, temperature_step, retention_time, ecl) in izip!(
            //     fatty_acid,
            //     onset_temperature,
            //     temperature_step,
            //     retention_time,
            //     ecl
            // ) {
            //     let Some(fatty_acid) = fatty_acid else {
            //         polars_bail!(NoData: "FattyAcid");
            //     };
            //     let Some(onset_temperature) = onset_temperature else {
            //         polars_bail!(NoData: "OnsetTemperature");
            //     };
            //     let Some(temperature_step) = temperature_step else {
            //         polars_bail!(NoData: "TemperatureStep");
            //     };
            //     let Some(retention_time) = retention_time else {
            //         polars_bail!(NoData: "RetentionTime");
            //     };
            //     let Some(ecl) = ecl else {
            //         polars_bail!(NoData: "ECL");
            //     };
            //     for (onset_temperature, temperature_step, retention_time, ecl) in izip!(
            //         onset_temperature.f64()?,
            //         temperature_step.list()?,
            //         retention_time.list()?,
            //         ecl.list()?
            //     ) {
            //         let Some(onset_temperature) = onset_temperature else {
            //             polars_bail!(NoData: "OnsetTemperature");
            //         };
            //         let Some(temperature_step) = temperature_step else {
            //             polars_bail!(NoData: "TemperatureStep");
            //         };
            //         let Some(retention_time) = retention_time else {
            //             polars_bail!(NoData: "RetentionTime");
            //         };
            //         let Some(ecl) = ecl else {
            //             polars_bail!(NoData: "ECL");
            //         };
            //         let mut points = Vec::new();
            //         for (temperature_step, retention_time, ecl) in
            //             izip!(temperature_step.f64()?, retention_time.f64()?, ecl.f64()?)
            //         {
            //             let Some(temperature_step) = temperature_step else {
            //                 polars_bail!(NoData: "TemperatureStep");
            //             };
            //             if let Some((retention_time, ecl)) = retention_time.zip(ecl) {
            //                 points.push([retention_time, ecl]);
            //             }
            //         }
            //         // Line
            //         let mut line = Line::new(points.clone())
            //             .name(format!("{:#}", (&fatty_acid).display(COMMON)));
            //         if fatty_acid.is_unsaturated()
            //             && self.settings.filter.mode.onset_temperature.is_none()
            //         {
            //             line = line.color(color(onset_temperature as _));
            //         } else {
            //             line = line.color(color(0 as _));
            //         };
            //         if fatty_acid.is_unsaturated() {
            //             line = line.style(LineStyle::Dashed { length: 16.0 });
            //         }
            //         ui.line(line);
            //         // // Points
            //         // let mut points = Points::new(points)
            //         //     .name(format!("{:#} {temperature_step}", (&fatty_acid).display(COMMON)))
            //         //     .color(color(onset_temperature as _))
            //         //     // .name(onset_temperature)
            //         //     // .color(color(onset_temperature as _))
            //         //     .radius(3.0);
            //         // if fatty_acid.unsaturation() == 0 {
            //         //     points = points.shape(MarkerShape::Square);
            //         // }
            //         // ui.points(points);
            //     }

            //     // if let Some(fatty_acid) = fatty_acid {
            //     //     points = points.name(format!("{:#}", (&fatty_acid).display(COMMON)));
            //     //     if fatty_acid.unsaturation() == 0 {
            //     //         points = points.shape(MarkerShape::Square).filled(false);
            //     //     }
            //     //     if fatty_acid.unsaturation() == 0 {
            //     //         points = points.shape(MarkerShape::Square).filled(false);
            //     //     }
            //     // }
            //     // let mut points = Points::new(points);
            //     //
            // }

            // let mut offsets = HashMap::new();
            // for (key, values) in visualized {
            //     // Bars
            //     let mut offset = 0.0;
            //     let x = key.into_inner();
            //     for (name, value) in values {
            //         let mut y = value;
            //         if percent {
            //             y *= 100.0;
            //         }
            //         let bar = Bar::new(x, y).name(name).base_offset(offset);
            //         let chart = BarChart::new(vec![bar])
            //             .width(context.settings.visualization.width)
            //             .name(x)
            //             .color(color(x as _));
            //         ui.bar_chart(chart);
            //         offset += y;
            //     }
            //     // // Text
            //     // if context.settings.visualization.text.show
            //     //     && offset >= context.settings.visualization.text.min
            //     // {
            //     //     let y = offset;
            //     //     let text = Text::new(
            //     //         PlotPoint::new(x, y),
            //     //         RichText::new(format!("{y:.p$}"))
            //     //             .size(context.settings.visualization.text.size)
            //     //             .heading(),
            //     //     )
            //     //     .name(x)
            //     //     .color(color(x as _))
            //     //     .anchor(Align2::CENTER_BOTTOM);
            //     //     ui.text(text);
            //     // }
            // }
            Ok(())
        });
        Ok(())
    }
}
