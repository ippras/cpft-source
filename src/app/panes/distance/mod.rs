use self::{settings::Settings, state::State, table::TableView};
use crate::{
    app::computers::{DistanceComputed, DistanceFilterComputed, DistanceFilterKey, DistanceKey},
    utils::save,
};
use egui::{CursorIcon, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, EXCLUDE, FLOPPY_DISK, GEAR};
use metadata::MetaDataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;

const ID_SOURCE: &str = "Source";

/// Distance pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: MetaDataFrame,
    pub(crate) target: DataFrame,
    pub(crate) settings: Settings,
    state: State,
}

impl Pane {
    pub(crate) fn new(frame: MetaDataFrame) -> Self {
        Self {
            source: frame,
            target: DataFrame::empty(),
            settings: Settings::new(),
            state: State::new(),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        EXCLUDE
    }

    pub(crate) fn title(&self) -> String {
        self.source.meta.title()
    }

    pub(super) fn header(&mut self, ui: &mut Ui) -> Response {
        ui.visuals_mut().button_frame = false;
        let mut response = ui
            .heading(Self::icon())
            .on_hover_text(ui.localize("distance"));
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
        .on_hover_text(ui.localize("resize"));
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.open_settings_window,
            RichText::new(GEAR).heading(),
        );
        ui.separator();
        // Save
        let name = format!("{}.distance.ipc", self.source.meta.title());
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
            memory.caches.cache::<DistanceComputed>().get(DistanceKey {
                data_frame: &self.source.data,
                settings: &self.settings,
            })
        });
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<DistanceFilterComputed>()
                .get(DistanceFilterKey {
                    data_frame: &self.target,
                    settings: &self.settings,
                })
        });
        TableView::new(&data_frame, &self.settings, &mut self.state).show(ui);
    }

    fn window(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Distance settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .open(&mut self.state.open_settings_window)
            .show(ui.ctx(), |ui| {
                self.settings.show(ui, &self.source.data);
            });
    }

    pub(super) fn hash(&self) -> u64 {
        hash(&self.source)
    }
}

pub(crate) mod settings;

mod state;
mod table;
