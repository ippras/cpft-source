use super::table::LEN;
use crate::{
    app::{
        MAX_PRECISION,
        panes::source::settings::{Axis, Filter, Order, PlotSettings, View},
    },
    localization::Text,
};
use egui::{ComboBox, Grid, RichText, Slider, Ui};
use egui_ext::LabeledSeparator;
use egui_l20n::{ResponseExt, UiExt as _};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) sort: Sort,
    pub(crate) filter: Filter,

    pub(crate) view: View,
    pub(crate) plot: PlotSettings,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            precision: 2,
            resizable: false,
            sticky: 0,
            truncate: false,
            sort: Sort::new(),
            filter: Filter::new(),
            view: View::Table,
            plot: PlotSettings::new(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) {
        Grid::new("Calculation").show(ui, |ui| -> PolarsResult<()> {
            // Precision floats
            ui.label(ui.localize("precision"));
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // Sticky columns
            ui.label(ui.localize("sticky"));
            ui.add(Slider::new(&mut self.sticky, 0..=LEN));
            ui.end_row();

            // Truncate titles
            ui.label(ui.localize("truncate"));
            ui.checkbox(&mut self.truncate, "");
            ui.end_row();

            // Filter
            ui.separator();
            ui.labeled_separator(RichText::new(ui.localize("filter")).heading());
            ui.end_row();

            self.filter.show(ui, data_frame)?;
            ui.end_row();

            // Sort
            ui.separator();
            ui.labeled_separator(RichText::new(ui.localize("sort-by-distance")).heading());
            ui.end_row();

            self.sort.show(ui);
            ui.end_row();

            if let View::Plot = self.view {
                // Plot
                ui.separator();
                ui.labeled_separator(RichText::new("Plot").heading());
                ui.end_row();

                // Legend
                ui.label(ui.localize("legend"));
                ui.checkbox(&mut self.plot.legend, "");
                ui.end_row();

                // Radius of points
                ui.label(ui.localize("radius-of-points"))
                    .on_hover_localized("radius-of-points.hover");
                ui.add(Slider::new(&mut self.plot.radius_of_points, 0..=u8::MAX).logarithmic(true));
                ui.end_row();

                // Plot axes
                for axis in [&mut self.plot.axes.x, &mut self.plot.axes.y] {
                    ui.label(ui.localize("plot-axes"));
                    ComboBox::from_id_salt(ui.next_auto_id())
                        .selected_text(ui.localize(axis.text()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(axis, Axis::Alpha, ui.localize(Axis::Alpha.text()))
                                .on_hover_localized(Axis::Alpha.hover_text());
                            ui.selectable_value(
                                axis,
                                Axis::EquivalentChainLength,
                                ui.localize(Axis::EquivalentChainLength.text()),
                            )
                            .on_hover_localized(Axis::EquivalentChainLength.hover_text());
                            ui.selectable_value(
                                axis,
                                Axis::OnsetTemperature,
                                ui.localize(Axis::OnsetTemperature.text()),
                            )
                            .on_hover_localized(Axis::OnsetTemperature.hover_text());
                            ui.selectable_value(
                                axis,
                                Axis::TemperatureStep,
                                ui.localize(Axis::TemperatureStep.text()),
                            )
                            .on_hover_localized(Axis::TemperatureStep.hover_text());
                        })
                        .response
                        .on_hover_localized(axis.hover_text());
                    ui.end_row();
                }
            }
            Ok(())
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Sort {
    pub(crate) aggregation: Aggregation,
    pub(crate) by: SortBy,
    pub(crate) order: Order,
}

impl Sort {
    fn new() -> Self {
        Self {
            aggregation: Aggregation::Maximum,
            by: SortBy::Value,
            order: Order::Descending,
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("sort-by-distance"))
            .on_hover_localized("sort-by-distance.hover");
        ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(ui.localize(self.by.text()))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.by, SortBy::Key, ui.localize(SortBy::Key.text()))
                    .on_hover_localized(SortBy::Key.hover_text());
                ui.selectable_value(
                    &mut self.by,
                    SortBy::Value,
                    ui.localize(SortBy::Value.text()),
                )
                .on_hover_localized(SortBy::Value.hover_text());
            })
            .response
            .on_hover_localized(self.by.hover_text());
        ui.end_row();

        // Aggregation
        ui.label(ui.localize("sort-by-aggregation"))
            .on_hover_localized("sort-by-aggregation.hover");
        let enabled = self.by == SortBy::Value;
        ui.add_enabled_ui(enabled, |ui| {
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(ui.localize(self.aggregation.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.aggregation,
                        Aggregation::Maximum,
                        ui.localize(Aggregation::Maximum.text()),
                    )
                    .on_hover_localized(Aggregation::Maximum.hover_text());
                    ui.selectable_value(
                        &mut self.aggregation,
                        Aggregation::Median,
                        ui.localize(Aggregation::Median.text()),
                    )
                    .on_hover_localized(Aggregation::Median.hover_text());
                    ui.selectable_value(
                        &mut self.aggregation,
                        Aggregation::Minimum,
                        ui.localize(Aggregation::Minimum.text()),
                    )
                    .on_hover_localized(Aggregation::Minimum.hover_text());
                })
                .response
                .on_hover_localized(self.aggregation.hover_text());
        })
        .response
        .on_disabled_hover_text("Used only for sort by value");
        ui.end_row();

        // Order
        ui.label(ui.localize("order"));
        ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(ui.localize(self.order.text()))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.order,
                    Order::Ascending,
                    ui.localize(Order::Ascending.text()),
                )
                .on_hover_localized(Order::Ascending.hover_text());
                ui.selectable_value(
                    &mut self.order,
                    Order::Descending,
                    ui.localize(Order::Descending.text()),
                )
                .on_hover_localized(Order::Descending.hover_text());
            })
            .response
            .on_hover_localized(self.order.hover_text());
    }
}

/// Sort by
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum SortBy {
    Key,
    Value,
}

impl Text for SortBy {
    fn text(&self) -> &'static str {
        match self {
            Self::Key => "sort-by-key",
            Self::Value => "sort-by-value",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Key => "sort-by-key.hover",
            Self::Value => "sort-by-value.hover",
        }
    }
}

/// Sort aggregation
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Aggregation {
    #[default]
    Maximum,
    Median,
    Minimum,
}

impl Text for Aggregation {
    fn text(&self) -> &'static str {
        match self {
            Self::Maximum => "sort-by-maximum",
            Self::Median => "sort-by-median",
            Self::Minimum => "sort-by-minimum",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Maximum => "sort-by-maximum.hover",
            Self::Median => "sort-by-median.hover",
            Self::Minimum => "sort-by-minimum.hover",
        }
    }
}

// /// Interpolation
// #[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
// pub(crate) struct Interpolation {
//     pub(crate) onset_temperature: f64,
//     pub(crate) temperature_step: f64,
// }

// impl Interpolation {
//     pub fn new() -> Self {
//         Self {
//             onset_temperature: 0.0,
//             temperature_step: 0.0,
//         }
//     }
// }

// impl Hash for Interpolation {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.onset_temperature.ord().hash(state);
//         self.temperature_step.ord().hash(state);
//     }
// }

// /// Retention time settings
// #[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
// pub(crate) struct RetentionTime {
//     pub(crate) precision: usize,
//     pub(crate) units: TimeUnits,
// }

// impl RetentionTime {
//     pub(crate) fn format(self, value: f32) -> RetentionTimeFormat {
//         RetentionTimeFormat {
//             value,
//             precision: Some(self.precision),
//             units: self.units,
//         }
//     }
// }

// impl Default for RetentionTime {
//     fn default() -> Self {
//         Self {
//             precision: 2,
//             units: Default::default(),
//         }
//     }
// }

// #[derive(Clone, Copy, Debug, Default)]
// pub(crate) struct RetentionTimeFormat {
//     value: f32,
//     precision: Option<usize>,
//     units: TimeUnits,
// }

// impl RetentionTimeFormat {
//     pub(crate) fn precision(self, precision: Option<usize>) -> Self {
//         Self { precision, ..self }
//     }
// }

// impl Display for RetentionTimeFormat {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         let time = Time::new::<millisecond>(self.value as _);
//         let value = match self.units {
//             TimeUnits::Millisecond => time.get::<millisecond>(),
//             TimeUnits::Second => time.get::<second>(),
//             TimeUnits::Minute => time.get::<minute>(),
//         };
//         if let Some(precision) = self.precision {
//             write!(f, "{value:.precision$}")
//         } else {
//             write!(f, "{value}")
//         }
//     }
// }

// impl From<RetentionTimeFormat> for WidgetText {
//     fn from(value: RetentionTimeFormat) -> Self {
//         value.to_string().into()
//     }
// }

// /// Time units
// #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
// pub enum TimeUnits {
//     Millisecond,
//     #[default]
//     Second,
//     Minute,
// }

// impl TimeUnits {
//     pub fn abbreviation(&self) -> &'static str {
//         Units::from(*self).abbreviation()
//     }

//     pub fn singular(&self) -> &'static str {
//         Units::from(*self).singular()
//     }

//     pub fn plural(&self) -> &'static str {
//         Units::from(*self).plural()
//     }
// }

// impl From<TimeUnits> for Units {
//     fn from(value: TimeUnits) -> Self {
//         match value {
//             TimeUnits::Millisecond => Units::millisecond(millisecond),
//             TimeUnits::Second => Units::second(second),
//             TimeUnits::Minute => Units::minute(minute),
//         }
//     }
// }
