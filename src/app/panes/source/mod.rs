use self::{
    plot::PlotView,
    settings::{Kind, Settings},
    state::State,
    table::TableView,
};
use crate::{
    app::{
        computers::{SourceComputed, SourceKey},
        localize,
    },
    utils::save,
};
use egui::{Button, CursorIcon, Id, Response, RichText, Ui, Window, util::hash};
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CHART_BAR, EXCLUDE, FLOPPY_DISK, GEAR, TABLE,
};
use metadata::MetaDataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;

const ID_SOURCE: &str = "Source";

/// Source pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: MetaDataFrame,
    pub(crate) target: DataFrame,
    pub(crate) settings: Settings,
    state: State,
}

impl Pane {
    pub(crate) const fn new(frame: MetaDataFrame) -> Self {
        Self {
            source: frame,
            target: DataFrame::empty(),
            settings: Settings::new(),
            state: State::new(),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        TABLE
    }

    pub(crate) fn title(&self) -> String {
        self.source.meta.title()
    }

    pub(super) fn header(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui.heading(Self::icon()).on_hover_text(localize!("source"));
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.hash()))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .clicked()
        {
            self.state.reset_table_state = true;
        }
        ui.separator();
        // Resize
        ui.toggle_value(
            &mut self.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(localize!("resize"));
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.open_settings_window,
            RichText::new(GEAR).heading(),
        );
        ui.separator();
        // Kind
        match self.settings.kind {
            Kind::Plot => {
                if ui.button(RichText::new(TABLE).heading()).clicked() {
                    self.settings.kind = Kind::Table;
                }
            }
            Kind::Table => {
                if ui.button(RichText::new(CHART_BAR).heading()).clicked() {
                    self.settings.kind = Kind::Plot;
                }
            }
        };
        ui.separator();
        // Distance
        if ui
            .add_enabled(
                self.settings.kind == Kind::Table,
                Button::new(RichText::new(EXCLUDE).heading()),
            )
            .clicked()
        {
            ui.data_mut(|data| {
                data.insert_temp(
                    Id::new("Distance"),
                    MetaDataFrame::new(self.source.meta.clone(), self.target.clone()),
                )
            })
        }
        ui.separator();
        // Save
        let name = format!("{}.source.ipc", self.source.meta.title());
        if ui
            .button(RichText::new(FLOPPY_DISK).heading())
            .on_hover_text(&name)
            .clicked()
        {
            if let Err(error) = save(
                &name,
                MetaDataFrame::new(&self.source.meta, &mut self.target),
            ) {
                error!(%error);
            }
        }
        ui.separator();
        response
    }

    pub(super) fn body(&mut self, ui: &mut Ui) {
        self.window(ui);
        self.target = ui.memory_mut(|memory| {
            memory.caches.cache::<SourceComputed>().get(SourceKey {
                data_frame: &self.source.data,
                settings: &self.settings,
            })
        });
        match self.settings.kind {
            Kind::Plot => PlotView::new(&self.target, &self.settings).show(ui),
            Kind::Table => TableView::new(&self.target, &self.settings, &mut self.state).show(ui),
        };
    }

    fn window(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Source settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .open(&mut self.state.open_settings_window)
            .show(ui.ctx(), |ui| {
                if let Err(error) = self.settings.show(ui, &self.source.data) {
                    error!(%error);
                }
            });
    }

    pub(super) fn hash(&self) -> u64 {
        hash(&self.source)
    }
}

pub(crate) mod settings;

mod plot;
mod state;
mod table;
