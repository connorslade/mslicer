use std::f32::consts::TAU;

use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};
use egui::{Button, Ui};
use slicer::mesh::Mesh;
use tools::graphics_3d;

use crate::{
    app::{App, camera::Camera, config::render::Projection},
    generator_tool,
    ui::popup::{Popup, PopupApp},
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

    ui.centered_and_justified(|ui| {
        if ui.add_enabled(!slicing, Button::new("Generate")).clicked() {
            tool.camera = app.camera.clone();
            tool.meshes = app.project.models.iter().map(|x| x.mesh.clone()).collect();
            generator_tool!(app, tool);
        }
    });

    false
}

#[derive(Default, Clone)]
pub struct Graphics3D {
    camera: Camera,
    meshes: Vec<Mesh>,
}

impl Graphics3D {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let n = 30;
        progress.set_total(n);

        let platform_size = config.platform_size.map(|x| x.raw());
        let aspect = platform_size.x / platform_size.y;

        let mut camera = self.camera.clone();

        (0..n)
            .map(|i| {
                camera.angle.x += (n as f32).recip() * TAU;
                let view_projection =
                    camera.view_projection_matrix(Projection::Perspective, aspect);
                let light = camera.position(1.0);

                let layer = graphics_3d::render(
                    config,
                    self.meshes.iter(),
                    view_projection,
                    light,
                    i as u32,
                );
                progress.add_complete(1);

                layer
            })
            .collect()
    }
}
