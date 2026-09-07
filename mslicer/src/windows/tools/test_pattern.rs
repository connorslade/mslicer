use egui::{Button, ComboBox, DragValue, Ui, Widget};
use tools::test_pattern::Pattern;

use crate::{
    app::App,
    generator_tool,
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "Generates patterns for testing and debugging purposes.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Test Pattern", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.pattern_generator;

    grid("").show(ui, |ui| {
        ui.label("Pattern");
        ComboBox::new("pattern", "")
            .selected_text(tool.pattern.name())
            .show_ui(ui, |ui| {
                for pattern in Pattern::ALL {
                    ui.selectable_value(&mut tool.pattern, pattern, pattern.name());
                }
            });
        ui.end_row();

        ui.label("Layers");
        ui.horizontal(|ui| {
            DragValue::new(&mut tool.layers).range(1..=u32::MAX).ui(ui);
            ui.take_available_width();
        });
        ui.end_row();
    });

    ui.add_space(8.0);
    ui.centered_and_justified(|ui| {
        if ui.add_enabled(!slicing, Button::new("Generate")).clicked() {
            generator_tool!(app, tool);
        }
    });

    false
}
