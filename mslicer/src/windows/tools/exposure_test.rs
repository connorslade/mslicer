use std::thread;

use clone_macro::clone;
use common::{
    progress::{CombinedProgress, Progress},
    units::Milimeter,
};
use egui::{Button, DragValue, Ui, Widget};
use image::RgbaImage;
use nalgebra::Vector2;
use slicer::util::export;

use crate::{
    app::{App, slice_operation::SliceOperation},
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
    windows::Tab,
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
            DragValue::new(&mut tool.steps).range(1.0..=f32::MAX).ui(ui);

            ui.label(format!("({:.2}mm each)", tool.size.x / tool.steps as f32));
            ui.take_available_width();
        });
        ui.end_row();
    });
    ui.add_space(8.0);

    ui.centered_and_justified(|ui| {
        if ui.add_enabled(!slicing, Button::new("Generate")).clicked() {
            let (tool, config) = (tool.clone(), app.project.slice_config.clone());
            let operation = SliceOperation::new(Progress::new(), CombinedProgress::new());
            operation.add_preview_image(RgbaImage::new(128, 128)); // blank preview image

            thread::spawn(clone!([operation], move || {
                let mut file = export(&config, tool.generate(&config, &operation.progress));
                file.0.set_preview(&operation.preview_image());
                operation.add_result(&config, file);
            }));
            app.slice_operation.replace(operation);
            app.panels
                .focus_tab(Tab::SliceOperation, Vector2::new(700.0, 400.0));
        }
    });

    false
}
