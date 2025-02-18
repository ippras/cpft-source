use super::{ID_SOURCE, Settings, State};
use crate::app::panes::{MARGIN, widgets::float::FloatValue};
use egui::{Frame, Id, Margin, TextStyle, TextWrapMode, Ui};
use egui_l20n::{ResponseExt as _, UiExt as _};
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
const FA: Range<usize> = MODE.end..MODE.end + 2;
const DISTANCE: Range<usize> = FA.end..FA.end + 3;
pub(super) const LEN: usize = DISTANCE.end;

const TOP: &[Range<usize>] = &[INDEX, MODE, FA, DISTANCE];

/// Table view
#[derive(Debug)]
pub(crate) struct TableView<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
    state: &'a mut State,
}

impl<'a> TableView<'a> {
    pub(crate) const fn new(
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
                ui.heading(ui.localize("fatty_acid"));
            }
            (0, DISTANCE) => {
                ui.heading(ui.localize("distance"));
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
            (1, fatty_acid::FROM) => {
                ui.heading(ui.localize("from"));
            }
            (1, fatty_acid::TO) => {
                ui.heading(ui.localize("to"));
            }
            (1, distance::TIME) => {
                ui.heading(ui.localize("retention_time"));
            }
            (1, distance::ECL) => {
                ui.heading(ui.localize("equivalent_chain_length.abbreviation"))
                    .on_hover_localized("equivalent_chain_length");
            }
            (1, distance::EUCLIDEAN) => {
                ui.heading(ui.localize("euclidean_distance"))
                    .on_hover_localized("euclidean_distance.hover");
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
            (row, fatty_acid::FROM) => {
                let fatty_acids = self.data_frame["From"].fa();
                let fatty_acid = fatty_acids.get(row)?.unwrap();
                let text = format!("{:#}", fatty_acid.display(COMMON));
                ui.label(&text).on_hover_text(text);
            }
            (row, fatty_acid::TO) => {
                let fatty_acids = self.data_frame["To"].fa();
                let fatty_acid = fatty_acids.get(row)?.unwrap();
                let text = format!("{:#}", fatty_acid.display(COMMON));
                ui.label(&text).on_hover_text(text);
            }
            (row, distance::TIME) => {
                let retention_time = self.data_frame["RetentionTime"].struct_().unwrap();
                let distance = retention_time.field_by_name("Distance").unwrap();
                ui.add(
                    FloatValue::new(distance.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                )
                .on_hover_ui(|ui| {
                    let from = retention_time.field_by_name("From").unwrap();
                    let to = retention_time.field_by_name("To").unwrap();
                    ui.horizontal(|ui| {
                        ui.label(to.str_value(row).unwrap());
                        ui.label("-");
                        ui.label(from.str_value(row).unwrap());
                    });
                });
            }
            (row, distance::ECL) => {
                let ecl = self.data_frame["ECL"].struct_().unwrap();
                let distance = ecl.field_by_name("Distance").unwrap();
                ui.add(
                    FloatValue::new(distance.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                )
                .on_hover_ui(|ui| {
                    let from = ecl.field_by_name("From").unwrap();
                    let to = ecl.field_by_name("To").unwrap();
                    ui.horizontal(|ui| {
                        ui.label(to.str_value(row).unwrap());
                        ui.label("-");
                        ui.label(from.str_value(row).unwrap());
                    });
                });
            }
            (row, distance::EUCLIDEAN) => {
                let distance = self.data_frame["Distance"].f64().unwrap();
                ui.add(
                    FloatValue::new(distance.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            _ => {}
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

mod fatty_acid {
    use super::*;

    pub(super) const FROM: Range<usize> = FA.start..FA.start + 1;
    pub(super) const TO: Range<usize> = FROM.end..FROM.end + 1;
}

mod distance {
    use super::*;

    pub(super) const TIME: Range<usize> = DISTANCE.start..DISTANCE.start + 1;
    pub(super) const ECL: Range<usize> = TIME.end..TIME.end + 1;
    pub(super) const EUCLIDEAN: Range<usize> = ECL.end..ECL.end + 1;
}
