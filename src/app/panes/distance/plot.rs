use crate::app::{
    computers::{DistancePlotValue, plot::IndexKey},
    panes::source::settings::PlotSettings as Settings,
};
use egui::Ui;
use egui_ext::color;
use egui_l20n::UiExt;
use egui_plot::{AxisHints, Legend, Line, Plot, PlotPoint, PlotPoints, Points, VPlacement};
use itertools::Itertools;
use lipid::fatty_acid::display::{COMMON, DisplayWithOptions as _};
use polars::prelude::*;
use std::fmt::Write;
use tracing::error;

/// Plot view
#[derive(Clone)]
pub(crate) struct PlotView<'a> {
    pub(crate) value: DistancePlotValue,
    pub(crate) settings: &'a Settings,
}

impl<'a> PlotView<'a> {
    pub(crate) fn new(data: DistancePlotValue, settings: &'a Settings) -> Self {
        Self {
            value: data,
            settings,
        }
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
        let onset_temperature = ui.localize("onset-temperature");
        let temperature_step = ui.localize("temperature-step");
        let alpha = ui.localize("alpha");
        let index = self.value.index.clone();
        plot = plot
            // .x_axis_label(&temperature_step)
            .custom_x_axes(vec![
                AxisHints::new_x()
                    .label(&temperature_step)
                    .placement(VPlacement::Top),
                AxisHints::new_x().label(&onset_temperature),
                AxisHints::new_x().label("X"),
            ])
            .y_axis_label(&alpha)
            .label_formatter(move |name, &PlotPoint { x, y }| {
                let mut label = String::new();
                if !name.is_empty() {
                    writeln!(&mut label, "{name}").ok();
                }
                if let Some(index) = index.get(&IndexKey(PlotPoint::new(x, y))) {
                    for (key, value) in index {
                        writeln!(&mut label, "{key} = {value:?}").ok();
                    }
                }
                label
            });
        plot.show(ui, |ui| -> PolarsResult<()> {
            for ((rank, onset_temperature), points) in &self.value.onset_temperature {
                // Line
                // let first = std::cmp::min(data.fatty_acids[0], data.fatty_acids[1]);
                let [from, to] = &self.value.fatty_acids[rank];
                let name = format!("{:#}-{:#}", from.display(COMMON), to.display(COMMON),);
                let line = Line::new(PlotPoints::Borrowed(points))
                    .name(&name)
                    .color(color(onset_temperature.0 as _));
                ui.line(line);
                // Points
                let points = Points::new(PlotPoints::Borrowed(points))
                    .name(name)
                    .color(color(*rank as _))
                    .radius(self.settings.radius_of_points);
                ui.points(points);
            }
            // for ((temperature_step, rank), points) in &self.value.temperature_step {
            //     // Line
            //     // let first = std::cmp::min(data.fatty_acids[0], data.fatty_acids[1]);
            //     let [from, to] = &self.value.fatty_acids[rank];
            //     let name = format!("{:#}-{:#}", from.display(COMMON), to.display(COMMON),);
            //     let line = Line::new(PlotPoints::Borrowed(points))
            //         .name(&name)
            //         .color(color(*rank as _));
            //     ui.line(line);
            //     // Points
            //     // let points = Points::new(PlotPoints::Borrowed(points))
            //     //     .name(name)
            //     //     .color(color(rank as _))
            //     //     .radius(self.settings.radius_of_points);
            //     // ui.points(points);
            // }
            Ok(())
        });
        Ok(())
    }
}
