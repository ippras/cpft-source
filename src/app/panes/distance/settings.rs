use super::table::LEN;
use crate::{
    app::{
        MAX_PRECISION,
        computers::{DistanceUniqueComputed, DistanceUniqueKey},
        panes::source::settings::Order,
    },
    localization::Text,
    special::data_frame::DataFrameExt as _,
    utils::VecExt,
};
use egui::{
    ComboBox, Grid, PopupCloseBehavior, RichText, Slider, TextWrapMode, Ui, WidgetText,
    emath::Float,
};
use egui_ext::LabeledSeparator;
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{FUNNEL, FUNNEL_X};
use lipid::{
    fatty_acid::display::{COMMON, DisplayWithOptions as _},
    prelude::*,
};
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};
use uom::si::{
    f32::Time,
    time::{Units, millisecond, minute, second},
};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) sort: SortByDistance,
    pub(crate) order: Order,

    pub(crate) filter: Filter,
    pub(crate) interpolation: Interpolation,
    pub(crate) filter_onset_temperature: Option<i32>,
    pub(crate) filter_temperature_step: Option<i32>,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            precision: 2,
            resizable: false,
            sticky: 1,
            truncate: false,
            sort: SortByDistance::Ecl,
            order: Order::Descending,

            filter: Filter::new(),
            interpolation: Interpolation::new(),
            filter_onset_temperature: None,
            filter_temperature_step: None,
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

            // ui.label("Interpolation");
            ui.label(ui.localize("filter-by-onset-temperature"))
                .on_hover_localized("filter-by-onset-temperature.hover");
            ui.add(Slider::new(
                &mut self.interpolation.onset_temperature,
                data_frame.mode().onset_temperature_range(),
            ));
            ui.end_row();

            ui.label(ui.localize("filter-by-temperature-step"))
                .on_hover_localized("filter-by-temperature-step.hover");
            ui.add(Slider::new(
                &mut self.interpolation.temperature_step,
                data_frame.mode().temperature_step_range(),
            ));
            ui.end_row();

            ui.label(ui.localize("filter-by-fatty-acids"))
                .on_hover_localized("filter-by-fatty-acids.hover");
            self.filter.show(ui, data_frame)?;
            ui.end_row();

            // Sort
            ui.separator();
            ui.labeled_separator(RichText::new(ui.localize("sort-by-distance")).heading());
            ui.end_row();

            ui.label(ui.localize("sort-by-distance"))
                .on_hover_localized("sort-by-distance.hover");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(ui.localize(self.sort.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.sort,
                        SortByDistance::RetentionTime,
                        ui.localize(SortByDistance::RetentionTime.text()),
                    )
                    .on_hover_localized(SortByDistance::RetentionTime.hover_text());
                    ui.selectable_value(
                        &mut self.sort,
                        SortByDistance::Ecl,
                        ui.localize(SortByDistance::Ecl.text()),
                    )
                    .on_hover_localized(SortByDistance::Ecl.hover_text());
                    ui.selectable_value(
                        &mut self.sort,
                        SortByDistance::Euclidean,
                        ui.localize(SortByDistance::Euclidean.text()),
                    )
                    .on_hover_localized(SortByDistance::Euclidean.hover_text());
                })
                .response
                .on_hover_localized(self.sort.hover_text());
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
            ui.end_row();
            Ok(())
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Filter {
    pub(crate) fatty_acids: Vec<FattyAcid>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            fatty_acids: Vec::new(),
        }
    }
}

impl Filter {
    fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) -> PolarsResult<()> {
        let text = format_list_truncated!(
            self.fatty_acids
                .iter()
                .map(|fatty_acid| fatty_acid.display(COMMON)),
            2
        );
        let inner_response = ComboBox::from_id_salt("FattyAcidsFilter")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| -> PolarsResult<()> {
                let fatty_acids = data_frame["FattyAcid"]
                    .unique()?
                    .sort(Default::default())?
                    .fa();
                for index in 0..fatty_acids.len() {
                    let Some(fatty_acid) = fatty_acids.get(index)? else {
                        continue;
                    };
                    let checked = self.fatty_acids.contains(&fatty_acid);
                    let response = ui
                        .selectable_label(checked, format!("{:#}", (&fatty_acid).display(COMMON)));
                    if response.clicked() {
                        if checked {
                            self.fatty_acids.remove_by_value(&fatty_acid);
                        } else {
                            self.fatty_acids.push(fatty_acid);
                        }
                    }
                    response.context_menu(|ui| {
                        if ui.button(format!("{FUNNEL} Select all")).clicked() {
                            self.fatty_acids = fatty_acids.clone().into_iter().flatten().collect();
                            ui.close_menu();
                        }
                        if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                            self.fatty_acids = Vec::new();
                            ui.close_menu();
                        }
                    });
                }
                Ok(())
            });
        inner_response.inner.transpose()?;
        inner_response.response.on_hover_ui(|ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            ui.label(self.fatty_acids.len().to_string());
        });
        Ok(())
    }
}

/// Interpolation
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Interpolation {
    pub(crate) onset_temperature: f64,
    pub(crate) temperature_step: f64,
}

impl Interpolation {
    pub const fn new() -> Self {
        Self {
            onset_temperature: 0.0,
            temperature_step: 0.0,
        }
    }
}

impl Hash for Interpolation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.ord().hash(state);
        self.temperature_step.ord().hash(state);
    }
}

/// Sort by distance
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum SortByDistance {
    Ecl,
    Euclidean,
    RetentionTime,
}

impl Text for SortByDistance {
    fn text(&self) -> &'static str {
        match self {
            Self::Ecl => "sort-by-equivalent-chain-length-distance",
            Self::Euclidean => "sort-by-euclidean-distance",
            Self::RetentionTime => "sort-by-retention-time-distance",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Ecl => "sort-by-equivalent-chain-length-distance.hover",
            Self::Euclidean => "sort-by-euclidean-distance.hover",
            Self::RetentionTime => "sort-by-retention-time-distance.hover",
        }
    }
}

/// Retention time settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct RetentionTime {
    pub(crate) precision: usize,
    pub(crate) units: TimeUnits,
}

impl RetentionTime {
    pub(crate) fn format(self, value: f32) -> RetentionTimeFormat {
        RetentionTimeFormat {
            value,
            precision: Some(self.precision),
            units: self.units,
        }
    }
}

impl Default for RetentionTime {
    fn default() -> Self {
        Self {
            precision: 2,
            units: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RetentionTimeFormat {
    value: f32,
    precision: Option<usize>,
    units: TimeUnits,
}

impl RetentionTimeFormat {
    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Display for RetentionTimeFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let time = Time::new::<millisecond>(self.value as _);
        let value = match self.units {
            TimeUnits::Millisecond => time.get::<millisecond>(),
            TimeUnits::Second => time.get::<second>(),
            TimeUnits::Minute => time.get::<minute>(),
        };
        if let Some(precision) = self.precision {
            write!(f, "{value:.precision$}")
        } else {
            write!(f, "{value}")
        }
    }
}

impl From<RetentionTimeFormat> for WidgetText {
    fn from(value: RetentionTimeFormat) -> Self {
        value.to_string().into()
    }
}

/// Time units
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TimeUnits {
    Millisecond,
    #[default]
    Second,
    Minute,
}

impl TimeUnits {
    pub fn abbreviation(&self) -> &'static str {
        Units::from(*self).abbreviation()
    }

    pub fn singular(&self) -> &'static str {
        Units::from(*self).singular()
    }

    pub fn plural(&self) -> &'static str {
        Units::from(*self).plural()
    }
}

impl From<TimeUnits> for Units {
    fn from(value: TimeUnits) -> Self {
        match value {
            TimeUnits::Millisecond => Units::millisecond(millisecond),
            TimeUnits::Second => Units::second(second),
            TimeUnits::Minute => Units::minute(minute),
        }
    }
}
