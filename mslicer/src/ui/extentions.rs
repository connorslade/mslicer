use egui::{Response, Ui, Widget};

pub trait WidgetExt {
    fn add(self, ui: &mut Ui) -> Response;
}

impl<T: Widget> WidgetExt for T {
    fn add(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}
