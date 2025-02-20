use crate::{
    app::MAX_PRECISION, localization::Text, special::data_frame::DataFrameExt as _,
    utils::VecExt as _,
};
use egui::{ComboBox, Grid, PopupCloseBehavior, RichText, Slider, TextWrapMode, Ui, emath::Float};
use egui_ext::LabeledSeparator;
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{FUNNEL, FUNNEL_X};
use lipid::{
    fatty_acid::display::{COMMON, DisplayWithOptions as _},
    prelude::*,
};
use polars::prelude::*;
use polars_utils::{format_list_container_truncated, format_list_truncated};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) kind: Kind,
    pub(crate) ddof: u8,
    pub(crate) logarithmic: bool,
    pub(crate) relative: Option<FattyAcid>,
    pub(crate) filter: Filter,
    pub(crate) sort: SortBy,
    pub(crate) order: Order,

    // pub(crate) group: Group,
    pub(crate) legend: bool,
    pub(crate) radius_of_points: u8,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            precision: 2,
            resizable: false,
            sticky: 1,
            truncate: false,

            kind: Kind::Table,
            ddof: 1,
            logarithmic: false,
            relative: None,
            filter: Filter::new(),
            sort: SortBy::Time,
            order: Order::Ascending,

            // group: Group::FattyAcid,
            radius_of_points: 2,
            legend: true,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) -> PolarsResult<()> {
        Grid::new("calculation")
            .show(ui, |ui| {
                // Precision floats
                ui.label(ui.localize("precision"));
                ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
                ui.end_row();

                // Sticky columns
                ui.label(ui.localize("sticky"));
                ui.add(Slider::new(&mut self.sticky, 0..=data_frame.width()));
                ui.end_row();

                // Truncate titles
                ui.label(ui.localize("truncate"));
                ui.checkbox(&mut self.truncate, "");
                ui.end_row();

                // Calculate
                ui.separator();
                ui.labeled_separator(RichText::new(ui.localize("calculate")).heading());
                ui.end_row();

                // Relative
                ui.label(ui.localize("relative-fatty-acid"))
                    .on_hover_localized("relative-fatty-acid.hover");
                ui.horizontal(|ui| {
                    let selected_text = self
                        .relative
                        .as_ref()
                        .map(|relative| relative.display(COMMON).to_string())
                        .unwrap_or_default();
                    ComboBox::from_id_salt(ui.auto_id_with("Relative"))
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| -> PolarsResult<()> {
                            let current_value = &mut self.relative;
                            let fatty_acid = data_frame["FattyAcid"].unique()?;
                            let fatty_acid = fatty_acid
                                .filter(&fatty_acid.fa().saturated_filter()?)?
                                .sort(Default::default())?
                                .fa();
                            for index in 0..fatty_acid.len() {
                                if let Some(selected_value) = fatty_acid.get(index)? {
                                    let text = (&selected_value).display(COMMON).to_string();
                                    ui.selectable_value(current_value, Some(selected_value), text);
                                }
                            }
                            Ok(())
                        })
                        .inner
                        .transpose()
                })
                .inner?;
                ui.end_row();

                // DDOF
                // https://numpy.org/devdocs/reference/generated/numpy.std.html
                ui.label(ui.localize("delta-degrees-of-freedom.abbreviation"))
                    .on_hover_localized("delta-degrees-of-freedom")
                    .on_hover_ui(|ui| {
                        ui.hyperlink(
                            "https://numpy.org/devdocs/reference/generated/numpy.std.html",
                        );
                    });
                ui.add(Slider::new(&mut self.ddof, 0..=2));
                ui.end_row();

                // Logarithmic
                ui.label(ui.localize("logarithmic"));
                ui.checkbox(&mut self.logarithmic, "");
                ui.end_row();

                // Filter
                ui.separator();
                ui.labeled_separator(RichText::new(ui.localize("filter")).heading());
                ui.end_row();

                self.filter.show(ui, data_frame)?;
                ui.end_row();

                // Sort
                ui.separator();
                ui.labeled_separator(RichText::new(ui.localize("sort-by")).heading());
                ui.end_row();

                ui.label(ui.localize("sort-by"))
                    .on_hover_localized("sort-by.hover");
                ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(ui.localize(self.sort.text()))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.sort,
                            SortBy::FattyAcid,
                            ui.localize(SortBy::FattyAcid.text()),
                        )
                        .on_hover_localized(SortBy::FattyAcid.hover_text());
                        ui.selectable_value(
                            &mut self.sort,
                            SortBy::Time,
                            ui.localize(SortBy::Time.text()),
                        )
                        .on_hover_localized(SortBy::Time.hover_text());
                    })
                    .response
                    .on_hover_localized(self.sort.hover_text());
                ui.end_row();

                // Order
                ui.label(ui.localize("order"))
                    .on_hover_localized("order.hover");
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

                if let Kind::Plot = self.kind {
                    // Plot
                    ui.separator();
                    ui.labeled_separator(RichText::new("Plot").heading());
                    ui.end_row();

                    // // Group
                    // ui.label("Group");
                    // ComboBox::from_id_salt(ui.next_auto_id())
                    //     .selected_text(self.group.text())
                    //     .show_ui(ui, |ui| {
                    //         ui.selectable_value(
                    //             &mut self.group,
                    //             Group::FattyAcid,
                    //             Group::FattyAcid.text(),
                    //         )
                    //         .on_hover_text(Group::FattyAcid.hover_text());
                    //         ui.selectable_value(
                    //             &mut self.group,
                    //             Group::OnsetTemperature,
                    //             Group::OnsetTemperature.text(),
                    //         )
                    //         .on_hover_text(Group::OnsetTemperature.hover_text());
                    //         ui.selectable_value(
                    //             &mut self.group,
                    //             Group::TemperatureStep,
                    //             Group::TemperatureStep.text(),
                    //         )
                    //         .on_hover_text(Group::TemperatureStep.hover_text());
                    //     })
                    //     .response
                    //     .on_hover_text(self.group.hover_text());
                    // ui.end_row();

                    // Legend
                    ui.label(ui.localize("legend"));
                    ui.checkbox(&mut self.legend, "");
                    ui.end_row();

                    // Radius of points
                    ui.label(ui.localize("radius-of-points"))
                        .on_hover_localized("radius-of-points.hover");
                    ui.add(Slider::new(&mut self.radius_of_points, 0..=u8::MAX).logarithmic(true));
                    ui.end_row();
                }
                Ok(())
            })
            .inner
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Group
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Group {
    #[default]
    FattyAcid,
    OnsetTemperature,
    TemperatureStep,
}

impl Text for Group {
    fn text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "Fatty acid",
            Self::OnsetTemperature => "Onset temperature",
            Self::TemperatureStep => "Temperature step",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "Group by fatty acid",
            Self::OnsetTemperature => "Group by onset temperature",
            Self::TemperatureStep => "Group by temperature step",
        }
    }
}

/// Kind
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Kind {
    Plot,
    #[default]
    Table,
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Filter {
    // pub(crate) mode: Mode,
    pub(crate) fatty_acids: Vec<FattyAcid>,
    pub(crate) onset_temperatures: Vec<f64>,
    pub(crate) temperature_steps: Vec<f64>,
}

impl Filter {
    pub const fn new() -> Self {
        Self {
            fatty_acids: Vec::new(),
            onset_temperatures: Vec::new(),
            temperature_steps: Vec::new(),
        }
    }
}

impl Hash for Filter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fatty_acids.hash(state);
        for onset_temperature in &self.onset_temperatures {
            onset_temperature.ord().hash(state);
        }
        for temperature_step in &self.temperature_steps {
            temperature_step.ord().hash(state);
        }
    }
}

impl Filter {
    fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) -> PolarsResult<()> {
        // Onset temperature filter
        ui.label(ui.localize("filter-by-onset-temperature"))
            .on_hover_localized("filter-by-onset-temperature.hover");
        let text = format_list_truncated!(&self.onset_temperatures, 2);
        ComboBox::from_id_salt("OnsetTemperatureFilter")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| -> PolarsResult<()> {
                let onset_temperatures = data_frame.mode().onset_temperature()?.unique();
                for onset_temperature in onset_temperatures.iter().flatten() {
                    let checked = self.onset_temperatures.contains(&onset_temperature);
                    let response =
                        ui.selectable_label(checked, AnyValue::from(onset_temperature).to_string());
                    if response.clicked() {
                        if checked {
                            self.onset_temperatures.remove_by_value(&onset_temperature);
                        } else {
                            self.onset_temperatures.push(onset_temperature);
                        }
                    }
                    response.context_menu(|ui| {
                        if ui.button(format!("{FUNNEL} Select all")).clicked() {
                            self.onset_temperatures = onset_temperatures.iter().flatten().collect();
                            ui.close_menu();
                        }
                        if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                            self.onset_temperatures = Vec::new();
                            ui.close_menu();
                        }
                    });
                }
                Ok(())
            })
            .inner
            .transpose()?;
        ui.end_row();

        // Temperature step filter
        ui.label(ui.localize("filter-by-temperature-step"))
            .on_hover_localized("filter-by-temperature-step.hover");
        let text = format_list_truncated!(&self.temperature_steps, 2);
        ComboBox::from_id_salt("TemperatureStepFilter")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| -> PolarsResult<()> {
                let temperature_steps = data_frame.mode().temperature_step()?.unique();
                for temperature_step in temperature_steps.iter().flatten() {
                    let checked = self.temperature_steps.contains(&temperature_step);
                    let response =
                        ui.selectable_label(checked, AnyValue::from(temperature_step).to_string());
                    if response.clicked() {
                        if checked {
                            self.temperature_steps.remove_by_value(&temperature_step);
                        } else {
                            self.temperature_steps.push(temperature_step);
                        }
                    }
                    response.context_menu(|ui| {
                        if ui.button(format!("{FUNNEL} Select all")).clicked() {
                            self.temperature_steps = temperature_steps.iter().flatten().collect();
                            ui.close_menu();
                        }
                        if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                            self.temperature_steps = Vec::new();
                            ui.close_menu();
                        }
                    });
                }
                Ok(())
            })
            .inner
            .transpose()?;
        ui.end_row();

        // Fatty acids filter
        ui.label(ui.localize("filter-by-fatty-acids"))
            .on_hover_localized("filter-by-fatty-acids.hover");
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

/// Sort by
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum SortBy {
    FattyAcid,
    Time,
}

impl Text for SortBy {
    fn text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "sort-by-fatty-acids",
            Self::Time => "sort-by-retention-time",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "sort-by-fatty-acids.hover",
            Self::Time => "sort-by-retention-time.hover",
        }
    }
}

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Order {
    Ascending,
    Descending,
}

impl Text for Order {
    fn text(&self) -> &'static str {
        match self {
            Self::Ascending => "ascending-order",
            Self::Descending => "descending-order",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Ascending => "ascending-order.hover",
            Self::Descending => "descending-order.hover",
        }
    }
}
