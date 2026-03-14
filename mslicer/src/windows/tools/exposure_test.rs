use std::thread;

use clone_macro::clone;
use common::progress::{CombinedProgress, Progress};
use egui::{Button, DragValue, Ui, Widget};
use image::RgbaImage;
use slicer::util::export;

use crate::{
    app::{App, slice_operation::SliceOperation},
    ui::{
        components::{grid, vec3_dragger},
        popup::{Popup, PopupApp},
    },
};

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Exposure Test", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    grid("").show(ui, |ui| {
        ui.label("Size (layers)");
        vec3_dragger(ui, app.state.exposure_test.size.as_mut(), |x| {
            x.fixed_decimals(0)
        });
        ui.end_row();

        ui.label("Steps");
        ui.horizontal(|ui| {
            DragValue::new(&mut 32).fixed_decimals(0).ui(ui);
            ui.take_available_width();
        });
        ui.end_row();
    });

    let slicing = (app.slice_operation.as_ref())
        .map(|x| !x.progress.complete())
        .unwrap_or_default();
    ui.centered_and_justified(|ui| {
        if ui.add_enabled(!slicing, Button::new("Generate")).clicked() {
            let config = app.project.slice_config.clone();
            let tool = app.state.exposure_test.clone();

            let operation = SliceOperation::new(Progress::new(), CombinedProgress::new());
            operation.add_preview_image(RgbaImage::new(128, 128)); // blank preview image

            thread::spawn(clone!([operation], move || {
                let file = export(&config, tool.generate(&config, &operation.progress));
                operation.add_result(&config, file);

                operation.progress.set_total(1);
                operation.progress.set_finished();
            }));
            app.slice_operation.replace(operation);
        }
    });

    false
}
