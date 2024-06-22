use eframe::Frame;
use egui::{Context, DragValue, Grid, Ui, Window};
use slicer::slicer::ExposureConfig;

use crate::{
    app::App,
    components::{vec2_dragger, vec3_dragger},
};

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    Window::new("Slice Config")
        .open(&mut app.windows.show_slice_config)
        .show(ctx, |ui| {
            Grid::new("slice_config")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Platform Resolution");
                    vec2_dragger::<u32>(ui, app.slice_config.platform_resolution.as_mut());
                    ui.end_row();

                    ui.label("Platform Size");
                    vec3_dragger::<f32>(ui, app.slice_config.platform_size.as_mut());
                    ui.end_row();

                    ui.label("Slice Height");
                    ui.add(DragValue::new(&mut app.slice_config.slice_height));
                    ui.end_row();

                    ui.label("First Layers");
                    ui.add(DragValue::new(&mut app.slice_config.first_layers));
                    ui.end_row();
                });

            ui.collapsing("Exposure Config", |ui| {
                exposure_config_grid(ui, &mut app.slice_config.exposure_config);
            });

            ui.collapsing("First Exposure Config", |ui| {
                exposure_config_grid(ui, &mut app.slice_config.first_exposure_config);
            });
        });
}

fn exposure_config_grid(ui: &mut Ui, config: &mut ExposureConfig) {
    Grid::new("exposure_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Exposure Time (s)");
            ui.add(DragValue::new(&mut config.exposure_time).clamp_range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Lift Distance (mm)");
            ui.add(DragValue::new(&mut config.lift_distance).clamp_range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Lift Speed (cm/min)");
            ui.add(DragValue::new(&mut config.lift_speed).clamp_range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Retract Distance (mm)");
            ui.add(DragValue::new(&mut config.retract_distance).clamp_range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Retract Speed (cm/min)");
            ui.add(DragValue::new(&mut config.retract_speed).clamp_range(0.0..=f32::MAX));
            ui.end_row();
        });
}
