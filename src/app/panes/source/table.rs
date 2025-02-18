use super::{ID_SOURCE, Settings, State};
use crate::app::panes::{MARGIN, widgets::float::FloatValue};
use egui::{Frame, Grid, Id, Margin, TextStyle, TextWrapMode, Ui};
use egui_l20n::{ResponseExt, UiExt};
use egui_phosphor::regular::HASH;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use lipid::{
    fatty_acid::display::{COMMON, DisplayWithOptions as _},
    prelude::*,
};
use polars::prelude::*;
use std::ops::Range;

const INDEX: Range<usize> = 0..1;
const MODE: Range<usize> = INDEX.end..INDEX.end + 2;
const FA: Range<usize> = MODE.end..MODE.end + 1;
const RETENTION_TIME: Range<usize> = FA.end..FA.end + 3;
const TEMPERATURE: Range<usize> = RETENTION_TIME.end..RETENTION_TIME.end + 1;
const CHAIN_LENGTH: Range<usize> = TEMPERATURE.end..TEMPERATURE.end + 3;
const MASS: Range<usize> = CHAIN_LENGTH.end..CHAIN_LENGTH.end + 1;
const DERIVATIVE: Range<usize> = MASS.end..MASS.end + 2;
const LEN: usize = DERIVATIVE.end;

const TOP: &[Range<usize>] = &[
    INDEX,
    MODE,
    FA,
    RETENTION_TIME,
    TEMPERATURE,
    CHAIN_LENGTH,
    MASS,
    DERIVATIVE,
];

/// Table view
#[derive(Debug)]
pub(super) struct TableView<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
    state: &'a mut State,
}

impl<'a> TableView<'a> {
    pub(super) const fn new(
        data_frame: &'a DataFrame,
        settings: &'a Settings,
        state: &'a mut State,
    ) -> Self {
        Self {
            data_frame,
            settings,
            state,
        }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.reset_table_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as _;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky)
            .headers([
                HeaderRow {
                    height,
                    groups: TOP.to_vec(),
                },
                HeaderRow::new(height),
            ])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, INDEX) => {
                ui.heading(HASH).on_hover_localized("index");
            }
            (0, MODE) => {
                ui.heading(ui.localize("mode"))
                    .on_hover_localized("mode.hover");
            }
            (0, FA) => {
                ui.heading(ui.localize("fatty_acid.abbreviation"))
                    .on_hover_localized("fatty_acid");
            }
            (0, RETENTION_TIME) => {
                ui.heading(ui.localize("retention_time"))
                    .on_hover_localized("retention_time.hover");
            }
            (0, TEMPERATURE) => {
                ui.heading(ui.localize("temperature"))
                    .on_hover_localized("temperature.hover");
            }
            (0, CHAIN_LENGTH) => {
                ui.heading(ui.localize("chain_length"))
                    .on_hover_localized("chain_length.hover");
            }
            (0, MASS) => {
                ui.heading(ui.localize("mass"))
                    .on_hover_localized("mass.hover");
            }
            (0, DERIVATIVE) => {
                ui.heading(ui.localize("derivative"))
                    .on_hover_localized("derivative.hover");
            }
            // Bottom
            (1, mode::ONSET) => {
                ui.heading(ui.localize("mode-onset_temperature"))
                    .on_hover_localized("mode-onset_temperature.hover");
            }
            (1, mode::STEP) => {
                ui.heading(ui.localize("mode-temperature_step"))
                    .on_hover_localized("mode-temperature_step.hover");
            }
            (1, retention_time::ABSOLUTE) => {
                ui.heading(ui.localize("retention_time-absolute"))
                    .on_hover_localized("retention_time-absolute.hover");
            }
            (1, retention_time::RELATIVE) => {
                ui.heading(ui.localize("retention_time-relative"))
                    .on_hover_localized("retention_time-relative.hover");
            }
            (1, retention_time::DIFF) => {
                ui.heading(ui.localize("retention_time-difference"))
                    .on_hover_localized("retention_time-difference.hover");
            }
            (1, chain_length::ECL) => {
                ui.heading(ui.localize("equivalent_chain_length.abbreviation"))
                    .on_hover_localized("equivalent_chain_length");
            }
            (1, chain_length::FCL) => {
                ui.heading(ui.localize("fractional_chain_length.abbreviation"))
                    .on_hover_localized("fractional_chain_length");
            }
            (1, chain_length::ECN) => {
                ui.heading(ui.localize("equivalent_carbon_number.abbreviation"))
                    .on_hover_localized("equivalent_carbon_number");
            }
            (1, derivative::SLOPE) => {
                ui.heading(ui.localize("slope"));
            }
            (1, derivative::ANGLE) => {
                ui.heading(ui.localize("angle"));
            }
            _ => {}
        }
    }

    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        match (row, column) {
            (row, INDEX) => {
                ui.label(row.to_string());
            }
            (row, mode::ONSET) => {
                let mode = self.data_frame["Mode"].struct_()?;
                let onset_temperature = mode.field_by_name("OnsetTemperature")?;
                ui.label(onset_temperature.str_value(row)?);
            }
            (row, mode::STEP) => {
                let mode = self.data_frame["Mode"].struct_()?;
                let temperature_step = mode.field_by_name("TemperatureStep")?;
                ui.label(temperature_step.str_value(row)?);
            }
            (row, FA) => {
                let fatty_acids = self.data_frame.fa();
                let fatty_acid = fatty_acids.get(row)?.unwrap();
                let text = format!("{:#}", fatty_acid.display(COMMON));
                ui.label(&text).on_hover_text(&text);
            }
            (row, retention_time::ABSOLUTE) => {
                let retention_time = self.data_frame["RetentionTime"].struct_()?;
                let absolute = retention_time.field_by_name("Absolute")?;
                let absolute = absolute.struct_()?;
                let mean = absolute.field_by_name("Mean")?;
                ui.add(
                    FloatValue::new(mean.f64()?.get(row)).precision(Some(self.settings.precision)),
                )
                .on_hover_ui(|ui| {
                    let standard_deviation = absolute.field_by_name("StandardDeviation").unwrap();
                    ui.horizontal(|ui| {
                        ui.label(mean.str_value(row).unwrap());
                        ui.label("±");
                        ui.label(standard_deviation.str_value(row).unwrap());
                    });
                })
                .on_hover_ui(|ui| {
                    ui.heading("Repeats");
                    let values = absolute
                        .field_by_name("Values")
                        .unwrap()
                        .list()
                        .unwrap()
                        .get_as_series(row)
                        .unwrap();
                    ui.vertical(|ui| {
                        for value in values.iter() {
                            ui.label(value.to_string());
                        }
                    });
                });
            }
            (row, retention_time::RELATIVE) => {
                let retention_time = self.data_frame["RetentionTime"].struct_()?;
                let relative = retention_time.field_by_name("Relative")?;
                ui.add(
                    FloatValue::new(relative.f64()?.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, retention_time::DIFF) => {
                let retention_time = self.data_frame["RetentionTime"].struct_()?;
                let delta = retention_time.field_by_name("Delta")?;
                ui.add(
                    FloatValue::new(delta.f64()?.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, TEMPERATURE) => {
                let temperature = &self.data_frame["Temperature"];
                ui.add(
                    FloatValue::new(temperature.f64()?.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, chain_length::ECL) => {
                let chain_length = self.data_frame["ChainLength"].struct_()?;
                let ecl = chain_length.field_by_name("ECL")?;
                ui.add(
                    FloatValue::new(ecl.f64()?.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, chain_length::FCL) => {
                let chain_length = self.data_frame["ChainLength"].struct_()?;
                let fcl = chain_length.field_by_name("FCL")?;
                ui.add(
                    FloatValue::new(fcl.f64()?.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, chain_length::ECN) => {
                let chain_length = self.data_frame["ChainLength"].struct_()?;
                let ecn = chain_length.field_by_name("ECN")?;
                ui.label(ecn.str_value(row)?);
            }
            (row, MASS) => {
                let mass = self.data_frame["Mass"].struct_()?;
                let rcooch3 = mass.field_by_name("RCOOCH3")?;
                ui.add(
                    FloatValue::new(rcooch3.f64()?.get(row))
                        .precision(Some(self.settings.precision)),
                )
                .on_hover_ui(|ui| {
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        ui.label("[RCO]+");
                        let rcoo = mass.field_by_name("RCO").unwrap();
                        ui.label(rcoo.str_value(row).unwrap());
                        ui.end_row();

                        ui.label("[RCOO]-");
                        let rcoo = mass.field_by_name("RCOO").unwrap();
                        ui.label(rcoo.str_value(row).unwrap());
                        ui.end_row();

                        ui.label("RCOOH");
                        let rcooh = mass.field_by_name("RCOOH").unwrap();
                        ui.label(rcooh.str_value(row).unwrap());
                        ui.end_row();

                        ui.label("RCOOCH3");
                        ui.label(rcooch3.str_value(row).unwrap());
                    });
                });
            }
            (row, derivative::SLOPE) => {
                let derivative = self.data_frame["Derivative"].struct_()?;
                let slope = derivative.field_by_name("Slope")?;
                ui.add(
                    FloatValue::new(slope.f64()?.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, derivative::ANGLE) => {
                let derivative = self.data_frame["Derivative"].struct_()?;
                let angle = derivative.field_by_name("Angle")?;
                ui.add(
                    FloatValue::new(angle.f64()?.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            _ => unreachable!(),
            // _ => {}
        }
        Ok(())
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.clone())
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 0 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1)
                    .unwrap()
            });
    }
}

mod mode {
    use super::*;

    pub(super) const ONSET: Range<usize> = MODE.start..MODE.start + 1;
    pub(super) const STEP: Range<usize> = ONSET.end..ONSET.end + 1;
}

mod retention_time {
    use super::*;

    pub(super) const ABSOLUTE: Range<usize> = RETENTION_TIME.start..RETENTION_TIME.start + 1;
    pub(super) const RELATIVE: Range<usize> = ABSOLUTE.end..ABSOLUTE.end + 1;
    pub(super) const DIFF: Range<usize> = RELATIVE.end..RELATIVE.end + 1;
}

mod chain_length {
    use super::*;

    pub(super) const ECL: Range<usize> = CHAIN_LENGTH.start..CHAIN_LENGTH.start + 1;
    pub(super) const FCL: Range<usize> = ECL.end..ECL.end + 1;
    pub(super) const ECN: Range<usize> = FCL.end..FCL.end + 1;
}

mod derivative {
    use super::*;

    pub(super) const SLOPE: Range<usize> = DERIVATIVE.start..DERIVATIVE.start + 1;
    pub(super) const ANGLE: Range<usize> = SLOPE.end..SLOPE.end + 1;
}
