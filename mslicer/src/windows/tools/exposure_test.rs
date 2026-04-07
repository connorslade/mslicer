use common::units::Milimeter;
use egui::{Button, DragValue, Ui, Widget};
use egui_phosphor::regular::{RULER, SQUARES_FOUR};

use crate::{
    app::App,
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "Generates a rectangular prism with a gradient of voxel vales across the top layer. Measuring the printed result will allow you get the value to voxel size mapping.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Exposure Test", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.exposure_test;
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
            ui.label("×");
            DragValue::new(&mut size[2])
                .max_decimals(2)
                .range(0.0..=platform.z)
                .ui(ui);
        });
        ui.end_row();

        ui.label("Steps");
        ui.horizontal(|ui| {
            DragValue::new(&mut tool.steps).range(1..=256).ui(ui);

            ui.label(format!("({:.2} mm each)", tool.size.x / tool.steps as f32));
            ui.take_available_width();
        });
        ui.end_row();

        ui.horizontal(|ui| {
            ui.label("Supports");
            ui.checkbox(&mut tool.supports.enabled, "");
        });
        ui.horizontal(|ui| {
            ui.add_enabled_ui(tool.supports.enabled, |ui| {
                DragValue::new(&mut tool.supports.height)
                    .range(0.1..=5.0)
                    .ui(ui);
                ui.label(RULER);

                ui.separator();
                DragValue::new(&mut tool.supports.spacing)
                    .range(0.1..=5.0)
                    .ui(ui);
                ui.label(SQUARES_FOUR);
            });
        });
        ui.end_row();
    });
    ui.add_space(8.0);

    ui.centered_and_justified(|ui| {
        if ui.add_enabled(!slicing, Button::new("Generate")).clicked() {
            crate::generator_tool!(app, tool);
        }
    });

    false
}
