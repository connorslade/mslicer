use eframe::Frame;
use egui::{Context, Slider, Window};

use crate::app::App;

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    Window::new("Transform")
        .open(&mut app.windows.show_transform)
        .default_width(0.0)
        .show(ctx, |ui| {
            ui.add(Slider::new(&mut app.camera.pos.x, -10.0..=10.0).text("X"));
            ui.add(Slider::new(&mut app.camera.pos.y, -10.0..=10.0).text("Y"));
            ui.add(Slider::new(&mut app.camera.pos.z, -10.0..=10.0).text("Z"));

            ui.separator();

            ui.add(
                Slider::new(
                    &mut app.camera.pitch,
                    -std::f32::consts::PI..=std::f32::consts::PI,
                )
                .text("Pitch"),
            );
            ui.add(
                Slider::new(
                    &mut app.camera.yaw,
                    -std::f32::consts::PI..=std::f32::consts::PI,
                )
                .text("Yaw"),
            );

            ui.separator();

            ui.add(Slider::new(&mut app.camera.fov, 0.0..=2.0 * std::f32::consts::PI).text("FOV"));
            ui.add(Slider::new(&mut app.camera.near, 0.0..=10.0).text("Near"));
            ui.add(Slider::new(&mut app.camera.far, 0.0..=100.0).text("Far"));
        });
}
