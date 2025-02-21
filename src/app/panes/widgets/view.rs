use crate::{app::panes::source::settings::View, localization::Text as _};
use egui::{Response, RichText, Ui, Widget};
use egui_l20n::{ResponseExt as _, UiExt as _};
use egui_phosphor::regular::{CHART_BAR, TABLE};

/// View widget
#[derive(Debug)]
pub(crate) struct ViewWidget<'a> {
    view: &'a mut View,
}

impl<'a> ViewWidget<'a> {
    pub(crate) fn new(view: &'a mut View) -> Self {
        Self { view }
    }
}

impl Widget for ViewWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text = match self.view {
            View::Plot => CHART_BAR,
            View::Table => TABLE,
        };
        ui.menu_button(RichText::new(text).heading(), |ui| {
            let mut response = ui
                .selectable_value(self.view, View::Table, ui.localize(View::Table.text()))
                .on_hover_localized(View::Table.hover_text());
            response |= ui
                .selectable_value(self.view, View::Plot, ui.localize(View::Plot.text()))
                .on_hover_localized(View::Plot.hover_text());
            if response.changed() {
                ui.close_menu();
            }
        })
        .response
    }
}
