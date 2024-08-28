use egui::{Context, DragValue, Grid, Ui};

use crate::{
    app::App,
    ui::components::{vec2_dragger, vec3_dragger},
};
use common::config::ExposureConfig;

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let width = ui.available_width();
    Grid::new("slice_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Platform Resolution");
            ui.horizontal(|ui| {
                vec2_dragger(ui, app.slice_config.platform_resolution.as_mut(), |x| x);
                ui.add_space(width);
            });
            ui.end_row();

            ui.label("Platform Size");
            vec3_dragger(ui, app.slice_config.platform_size.as_mut(), |x| x);
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

    ui.add_space(16.0);
    ui.heading("Plugins");

    let this_app = unsafe { &mut *(app as *mut _) };
    for plugin in &mut app.plugin_manager.plugins {
        ui.collapsing(plugin.name(), |ui| {
            plugin.ui(this_app, ui, _ctx);
        });
    }
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
