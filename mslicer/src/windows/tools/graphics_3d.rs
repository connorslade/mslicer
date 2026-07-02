use std::f32::consts::TAU;

use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};
use egui::{Button, Ui};
use nalgebra::Vector2;
use slicer::{
    mesh::Mesh,
    slicer::raster::{self, Segment},
};

use crate::{
    app::App,
    generator_tool,
    render::camera::{Camera, Projection},
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

        let platform = config.platform_resolution * config.supersample as u32;
        let platform_size = config.platform_size.map(|x| x.raw());
        let aspect = platform_size.x / platform_size.y;

        let mut camera = self.camera.clone();

        (0..n)
            .map(|i| {
                camera.angle.x += (n as f32).recip() * TAU;
                let view_projection =
                    camera.view_projection_matrix(Projection::Perspective, aspect);

                let mut faces = Vec::new();
                let mut min_depth = f32::INFINITY;
                let mut max_depth = f32::NEG_INFINITY;

                for mesh in self.meshes.iter() {
                    for (i, face) in mesh.faces().iter().enumerate() {
                        let verts = face
                            .map(|x| mesh.transform(&mesh.vertices()[x as usize]))
                            .map(|x| view_projection * x.push(1.0));
                        if verts.iter().any(|x| x.w < 0.1) {
                            continue;
                        }

                        let depth = (verts[0].w + verts[1].w + verts[2].w) / 3.0;
                        min_depth = min_depth.min(depth);
                        max_depth = max_depth.max(depth);

                        let [a, b, c] = verts.map(|x| x / x.w).map(|p| {
                            Vector2::new(
                                (p.x * 0.5 + 0.5) * platform.x as f32,
                                (1.0 - (p.y * 0.5 + 0.5)) * platform.y as f32,
                            )
                        });

                        let signed_area = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
                        if signed_area > 0.0 {
                            continue;
                        }

                        let normal = (view_projection
                            * mesh.transform_normal(&mesh.normal(i)).push(0.0))
                        .xyz();
                        let light = -camera.position(1.0);

                        let diffuse = normal.dot(&light).max(0.0);
                        let reflect_dir = (-light) - 2.0 * normal.dot(&(-light)) * normal;
                        let specular = light.dot(&reflect_dir).max(0.0).powi(32);
                        let intensity = ((diffuse + specular + 0.1) * 255.0) as u8;

                        faces.push((a, b, c, depth, intensity));
                    }
                }

                let mut segments = Vec::new();
                for (a, b, c, avg_depth, intensity) in faces {
                    let norm = (max_depth - avg_depth) / (max_depth - min_depth);
                    let depth = (1.0 + norm * 254.0) as u8;

                    for segment in [[a, b], [b, c], [c, a]] {
                        segments.push(Segment {
                            endpoints: segment,
                            entering: segment[1].y < segment[0].y,
                            priority: depth,
                            exposure: intensity,
                        });
                    }
                }

                let runs = raster::layer(
                    config.supersample,
                    config.platform_resolution,
                    segments.into_iter(),
                );
                progress.add_complete(1);

                Layer {
                    data: runs,
                    exposure: config.exposure_config(i as u32).into_owned(),
                }
            })
            .collect()
    }
}
