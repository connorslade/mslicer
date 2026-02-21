use egui::{Response, Ui, Widget};

use crate::ui::components::being_edited;

pub trait WidgetExt {
    fn add(self, ui: &mut Ui) -> Response;
}

pub trait ResposeExt {
    fn being_edited(&self) -> bool;
}

impl<T: Widget> WidgetExt for T {
    fn add(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl ResposeExt for Response {
    fn being_edited(&self) -> bool {
        being_edited(self)
    }
}
