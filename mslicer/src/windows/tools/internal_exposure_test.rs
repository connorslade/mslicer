use common::units::Milimeter;
use egui::{Button, DragValue, Ui, Widget};

use crate::{
    app::App,
    generator_tool,
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "Generates a rectangular prism with a gradient of voxel vales inside. Intended for use with translucent resins.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Internal Exposure Test", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.internal_exposure_test;
    grid("").show(ui, |ui| {
        ui.label("Size (mm)");

        let size = tool.size.as_mut();
        let platform = (app.project.slice_config.platform_size).map(|x| x.get::<Milimeter>());

        ui.horizontal(|ui| {
            DragValue::new(&mut size[0])
                .max_decimals(2)
                .range(0.0..=platform.x)
                .ui(ui);
            ui.label("×");
            DragValue::new(&mut size[1])
                .max_decimals(2)
                .range(0.0..=platform.y)
                .ui(ui);
        });
        ui.end_row();

        ui.label("Cavity");
        DragValue::new(&mut tool.wall).ui(ui);
        ui.end_row();

        ui.label("Border Exposure");
        ui.horizontal(|ui| {
            DragValue::new(&mut tool.border_exposure)
                .range(0.0..=1.0)
                .speed(0.01)
                .ui(ui);
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
