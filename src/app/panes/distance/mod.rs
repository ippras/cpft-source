use self::{settings::Settings, state::State, table::TableView};
use crate::{
    app::computers::{
        DistanceComputed, DistanceFilteredComputed, DistanceFilteredKey, DistanceKey,
    },
    utils::save,
};
use egui::{CursorIcon, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, EXCLUDE, FLOPPY_DISK, GEAR};
use metadata::MetaDataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;

const ID_SOURCE: &str = "Distance";

/// Distance pane
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
        EXCLUDE
    }

    pub(crate) fn title(&self) -> String {
        self.source.frame.meta.title()
    }

    pub(super) fn header(&mut self, ui: &mut Ui) -> Response {
        ui.visuals_mut().button_frame = false;
        let mut response = ui
            .heading(Self::icon())
            .on_hover_text(ui.localize("distance"));
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
        // Save
        let name = format!("{}.distance.ipc", self.source.frame.meta.title());
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
            memory.caches.cache::<DistanceComputed>().get(DistanceKey {
                data_frame: &self.source.frame.data,
                hash: self.source.hash,
            })
        });
        // Filtered
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<DistanceFilteredComputed>()
                .get(DistanceFilteredKey {
                    data_frame: &self.target,
                    settings: &self.settings,
                })
        });
        TableView::new(&data_frame, &self.settings, &mut self.state).show(ui);
    }

    fn window(&mut self, ui: &mut Ui) {
        Window::new(ui.localize("distance-settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .open(&mut self.state.open_settings_window)
            .show(ui.ctx(), |ui| {
                self.settings.show(ui, &self.source.frame.data);
            });
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Source {
    frame: MetaDataFrame,
    hash: u64,
}

pub(crate) mod settings;

mod state;
mod table;
