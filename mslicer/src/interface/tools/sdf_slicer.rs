use egui::{Button, Ui, vec2};

use crate::{
    core::App,
    generator_tool,
    interface::popup::{Popup, PopupApp},
};

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new_seeded("SDF Slicer", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label("Slice SDFs");
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.sdf_slicer;

    ui.add_space(8.0);
    ui.vertical_centered(|ui| {
        let button = Button::new("Generate").min_size(vec2(ui.available_width(), 0.0));
        if ui.add_enabled(!slicing, button).clicked() {
            generator_tool!(app, tool);
        }
    });

    false
}
