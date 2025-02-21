use self::{
    plot::PlotView,
    settings::{Settings, View},
    state::State,
    table::TableView,
};
use crate::{
    app::computers::{SourceComputed, SourceKey, SourcePlotComputed, SourcePlotKey},
    utils::save,
};
use egui::{Button, CursorIcon, Id, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, EXCLUDE, FLOPPY_DISK, GEAR, TABLE,
};
use metadata::MetaDataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;

use super::widgets::ViewWidget;

const ID_SOURCE: &str = "Source";

/// Source pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    source: Source,
    target: DataFrame,
    settings: Settings,
    state: State,
}

impl Pane {
    pub(crate) fn new(frame: MetaDataFrame) -> Self {
        let hash = hash(&frame);
        Self {
            source: Source { frame, hash },
            target: DataFrame::empty(),
            settings: Settings::new(),
            state: State::new(),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        TABLE
    }

    pub(crate) fn title(&self) -> String {
        self.source.frame.meta.title()
    }

    pub(super) fn header(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui
            .heading(Self::icon())
            .on_hover_text(ui.localize("source"));
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.source.hash))
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
        .on_hover_text(ui.localize("resize"));
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.open_settings_window,
            RichText::new(GEAR).heading(),
        );
        ui.separator();
        // View
        ui.add(ViewWidget::new(&mut self.settings.view));
        ui.separator();
        // Distance
        if ui
            .add_enabled(
                self.settings.view == View::Table,
                Button::new(RichText::new(EXCLUDE).heading()),
            )
            .clicked()
        {
            ui.data_mut(|data| {
                data.insert_temp(
                    Id::new("Distance"),
                    MetaDataFrame::new(self.source.frame.meta.clone(), self.target.clone()),
                )
            })
        }
        ui.separator();
        // Save
        let name = format!("{}.source.ipc", self.source.frame.meta.title());
        if ui
            .button(RichText::new(FLOPPY_DISK).heading())
            .on_hover_text(&name)
            .clicked()
        {
            if let Err(error) = save(
                &name,
                MetaDataFrame::new(&self.source.frame.meta, &mut self.target),
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
                data_frame: &self.source.frame.data,
                settings: &self.settings,
            })
        });
        match self.settings.view {
            View::Plot => {
                let points = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<SourcePlotComputed>()
                        .get(SourcePlotKey {
                            data_frame: &self.target,
                            settings: &self.settings,
                        })
                });
                PlotView::new(points, &self.settings).show(ui)
            }
            View::Table => TableView::new(&self.target, &self.settings, &mut self.state).show(ui),
        };
    }

    fn window(&mut self, ui: &mut Ui) {
        Window::new(ui.localize("source-settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .open(&mut self.state.open_settings_window)
            .show(ui.ctx(), |ui| {
                if let Err(error) = self.settings.show(ui, &self.source.frame.data) {
                    error!(%error);
                }
            });
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Source {
    frame: MetaDataFrame,
    hash: u64,
}

pub(crate) mod settings;

mod plot;
mod state;
mod table;
