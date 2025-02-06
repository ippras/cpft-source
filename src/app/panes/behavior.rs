use super::Pane;
use egui::{CentralPanel, RichText, Sides, TopBottomPanel, Ui, WidgetText};
use egui_phosphor::regular::X;
use egui_tiles::{TileId, UiResponse};
use serde::{Deserialize, Serialize};

/// Behavior
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Behavior {
    pub(crate) close: Option<TileId>,
    pub(crate) click: Option<TileId>,
}

impl egui_tiles::Behavior<Pane> for Behavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.title().into()
    }

    // fn tab_title_for_tile(&mut self, tiles: &Tiles<Pane>, tile_id: TileId) -> WidgetText {
    //     if let Some(tile) = tiles.get(tile_id) {
    //         match tile {
    //             Tile::Pane(pane) => self.tab_title_for_pane(pane),
    //             Tile::Container(container) => {
    //                 if let Some(pane) = container.find_child_pane(tiles) {
    //                     format!("{}, ...", self.tab_title_for_pane(pane).text()).into()
    //                 } else {
    //                     format!("{:?}", container.kind()).into()
    //                 }
    //             }
    //         }
    //     } else {
    //         "MISSING TILE".into()
    //     }
    // }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        let response = TopBottomPanel::top(ui.auto_id_with("TopPanel"))
            .show_inside(ui, |ui| {
                Sides::new()
                    .show(
                        ui,
                        |ui| pane.header(ui),
                        |ui| {
                            ui.visuals_mut().button_frame = false;
                            if ui.button(RichText::new(X).heading()).clicked() {
                                self.close = Some(tile_id);
                            }
                        },
                    )
                    .0
            })
            .inner;
        CentralPanel::default().show_inside(ui, |ui| {
            pane.body(ui);
        });
        if response.dragged() {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }
}
