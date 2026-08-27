use std::f32::consts::TAU;

use egui::{Button, DragValue, Ui, Widget};

use crate::{
    app::{App, config::render::Projection},
    generator_tool,
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "peak unemployment";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("3D Graphics", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.graphics_3d;

    grid("").show(ui, |ui| {
        ui.label("Angles");
        DragValue::new(&mut tool.angles).range(1..=u32::MAX).ui(ui);
        ui.end_row();
    });
    ui.add_space(8.0);

    ui.centered_and_justified(|ui| {
        if ui.add_enabled(!slicing, Button::new("Generate")).clicked() {
            tool.meshes = app.project.models.iter().map(|x| x.mesh.clone()).collect();

            let camera = app.camera.clone();
            let step = TAU / tool.angles.max(1) as f32;

            generator_tool!(app, tool, move |aspect: f32, i: u32| {
                let mut camera = camera.clone();
                camera.angle.x += step * (i + 1) as f32;

                let transform = camera.view_projection_matrix(Projection::Perspective, aspect);
                let light = camera.position(1.0);
                (transform, light)
            });
        }
    });

    false
}
