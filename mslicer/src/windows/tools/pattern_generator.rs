use egui::{Button, Ui};

use crate::{
    app::App,
    generator_tool,
    ui::popup::{Popup, PopupApp},
};

pub const DESCRIPTION: &str = "wip";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Pattern Generator", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.pattern_generator;

    ui.centered_and_justified(|ui| {
        if ui.add_enabled(!slicing, Button::new("Generate")).clicked() {
            generator_tool!(app, tool);
        }
    });

    false
}
