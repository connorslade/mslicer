use egui::{CollapsingHeader, Context, Ui};
use nalgebra::Vector3;
use slicer::supports::line::LineSupportGenerator;

use crate::{app::App, render::rendered_mesh::RenderedMesh, ui::components::dragger};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.label("Generate supports to allow printing overhangs in models. You can generate supports for individual meshes or all meshes at once.");

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let mut meshes = app.meshes.write();
        ui.menu_button("Generate", |ui| {
            ui.style_mut().visuals.button_frame = false;

            for idx in 0..meshes.len() {
                let mesh = &meshes[idx];
                if ui.button(&mesh.name).clicked() {
                    let generator = LineSupportGenerator::new(
                        &app.state.line_support_config,
                        app.slice_config.platform_size,
                    );

                    let debug = generate_support(&mut meshes, idx, &generator);
                    app.state.line_support_debug.extend_from_slice(&debug);
                }
            }
        });

        if ui.button("Generate All").clicked() {
            app.state.line_support_debug = Vec::new();
            let generator = LineSupportGenerator::new(
                &app.state.line_support_config,
                app.slice_config.platform_size,
            );

            for i in 0..meshes.len() {
                let debug = generate_support(&mut meshes, i, &generator);
                app.state.line_support_debug.extend_from_slice(&debug);
            }
        }
    });

    ui.add_space(8.0);
    ui.heading("Support Config");

    let support = &mut app.state.line_support_config;

    CollapsingHeader::new("Overhang Detection")
        .default_open(true)
        .show(ui, |ui| {
            dragger(
                ui,
                "Max Origin Normal Z",
                &mut support.max_origin_normal_z,
                |x| x.speed(0.01),
            );

            dragger(
                ui,
                "Max Neighbor Z Diff",
                &mut support.max_neighbor_z_diff,
                |x| x.speed(0.01),
            );

            dragger(ui, "Min Angle", &mut support.min_angle, |x| x.speed(0.01));
        });

    CollapsingHeader::new("Support Generation")
        .default_open(true)
        .show(ui, |ui| {
            dragger(
                ui,
                "Face Support Spacing",
                &mut support.face_support_spacing,
                |x| x.speed(0.1),
            );

            dragger(ui, "Support Radius", &mut support.support_radius, |x| {
                x.speed(0.1)
            });

            dragger(
                ui,
                "Support Precision",
                &mut support.support_precision,
                |x| x,
            );
        });
}

fn generate_support(
    meshes: &mut Vec<RenderedMesh>,
    idx: usize,
    support: &LineSupportGenerator,
) -> Vec<[Vector3<f32>; 2]> {
    let mesh = &meshes[idx];

    let (supports, debug) = support.generate_line_supports(&mesh.mesh);

    let mesh = RenderedMesh::from_mesh(supports)
        .with_name(format!("Supports {}", mesh.name))
        .with_random_color();

    meshes.push(mesh);

    debug
}
