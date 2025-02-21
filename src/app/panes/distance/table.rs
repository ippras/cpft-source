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
const DISTANCE: Range<usize> = FA.end..FA.end + 4;
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
                ui.heading(ui.localize("fatty-acid"))
                    .on_hover_localized("fatty-acid.abbreviation");
            }
            (0, DISTANCE) => {
                ui.heading(ui.localize("distance"));
            }
            // Bottom
            (1, mode::ONSET) => {
                ui.heading(ui.localize("onset-temperature.abbreviation"))
                    .on_hover_localized("onset-temperature");
            }
            (1, mode::STEP) => {
                ui.heading(ui.localize("temperature-step.abbreviation"))
                    .on_hover_localized("temperature-step")
                    .on_hover_localized("temperature-step.hover");
            }
            (1, fatty_acid::FROM) => {
                ui.heading(ui.localize("from"));
            }
            (1, fatty_acid::TO) => {
                ui.heading(ui.localize("to"));
            }
            (1, distance::RETENTION_TIME) => {
                ui.heading(ui.localize("retention-time-distance.abbreviation"))
                    .on_hover_localized("retention-time-distance")
                    .on_hover_localized("retention-time-distance.hover");
            }
            (1, distance::ECL) => {
                ui.heading(ui.localize("equivalent-chain-length-distance.abbreviation"))
                    .on_hover_localized("equivalent-chain-length-distance")
                    .on_hover_localized("equivalent-chain-length-distance.hover");
            }
            (1, distance::EUCLIDEAN) => {
                ui.heading(ui.localize("euclidean-distance.abbreviation"))
                    .on_hover_localized("euclidean-distance")
                    .on_hover_localized("euclidean-distance.hover");
            }
            (1, distance::ALPHA) => {
                ui.heading(ui.localize("alpha.abbreviation"))
                    .on_hover_localized("alpha")
                    .on_hover_localized("alpha.hover");
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
                let fatty_acid = self.data_frame["FattyAcid"]
                    .struct_()?
                    .field_by_name("From")?
                    .fa();
                let Some(from) = fatty_acid.get(row)? else {
                    polars_bail!(NoData: "FattyAcid/From[{row}]");
                };
                let text = format!("{:#}", from.display(COMMON));
                ui.label(&text).on_hover_text(text);
            }
            (row, fatty_acid::TO) => {
                let fatty_acid = self.data_frame["FattyAcid"]
                    .struct_()?
                    .field_by_name("To")?
                    .fa();
                let Some(to) = fatty_acid.get(row)? else {
                    polars_bail!(NoData: "FattyAcid/To[{row}]");
                };
                let text = format!("{:#}", to.display(COMMON));
                ui.label(&text).on_hover_text(text);
            }
            (row, distance::RETENTION_TIME) => {
                let retention_time = self.data_frame["RetentionTime"].struct_()?;
                let delta = retention_time.field_by_name("Delta")?;
                ui.add(
                    FloatValue::new(delta.f64()?.get(row))
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
                let ecl = self.data_frame["EquivalentChainLength"].struct_()?;
                let delta = ecl.field_by_name("Delta")?;
                ui.add(
                    FloatValue::new(delta.f64()?.get(row))
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
                let distance = self.data_frame["EuclideanDistance"].f64()?;
                ui.add(
                    FloatValue::new(distance.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, distance::ALPHA) => {
                let alpha = self.data_frame["Alpha"].f64()?;
                let response = ui.add(
                    FloatValue::new(alpha.get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
                response.on_hover_ui(|ui| {
                    (|| -> PolarsResult<()> {
                        // ui.spacing_mut().item_spacing.x = 0.0;
                        let retention_time = self.data_frame["RetentionTime"].struct_()?;
                        let dead_time = self.data_frame["DeadTime"].get(row)?.str_value();
                        let from = retention_time.field_by_name("From")?;
                        let to = retention_time.field_by_name("To")?;
                        let from = from.str_value(row)?;
                        let to = to.str_value(row)?;
                        ui.horizontal(|ui| {
                            let math =
                                format!(r#"$\frac{{{to}-{dead_time}}}{{{from}-{dead_time}}}$"#);
                            ui.label(math);
                            // ui.label(to);
                            // ui.label("-");
                            // ui.label(&*dead_time);
                            // ui.label("/");
                            // ui.label(from);
                            // ui.label("-");
                            // ui.label(dead_time);
                        });
                        Ok(())
                    })()
                    .unwrap()
                });
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

    pub(super) const RETENTION_TIME: Range<usize> = DISTANCE.start..DISTANCE.start + 1;
    pub(super) const ECL: Range<usize> = RETENTION_TIME.end..RETENTION_TIME.end + 1;
    pub(super) const EUCLIDEAN: Range<usize> = ECL.end..ECL.end + 1;
    pub(super) const ALPHA: Range<usize> = EUCLIDEAN.end..EUCLIDEAN.end + 1;
}
