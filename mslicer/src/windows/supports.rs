use egui::{Context, Ui};
use slicer::supports::line::LineSupportGenerator;

use crate::{app::App, render::rendered_mesh::RenderedMesh, ui::components::dragger};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    if ui.button("Generate").clicked() {
        app.state.line_support_debug = Vec::new();
        let generator = LineSupportGenerator::new(
            &app.state.line_support_config,
            app.slice_config.platform_size,
        );

        let mut meshes = app.meshes.write();

        for i in 0..meshes.len() {
            let mesh = &meshes[i];

            let supports = generator.generate_line_supports(&mesh.mesh);
            let mesh = RenderedMesh::from_mesh(supports)
                .with_name("Supports".into())
                .with_random_color();

            meshes.push(mesh);
        }
    }

    ui.add_space(16.0);
    ui.heading("Support Config");

    dragger(
        ui,
        "Max Origin Normal Z",
        &mut app.state.line_support_config.max_origin_normal_z,
        |x| x.speed(0.01),
    );

    dragger(
        ui,
        "Max Neighbor Z Diff",
        &mut app.state.line_support_config.max_neighbor_z_diff,
        |x| x.speed(0.01),
    );

    dragger(
        ui,
        "Min Angle",
        &mut app.state.line_support_config.min_angle,
        |x| x.speed(0.01),
    );

    dragger(
        ui,
        "Face Support Spacing",
        &mut app.state.line_support_config.face_support_spacing,
        |x| x.speed(0.1),
    );
}
