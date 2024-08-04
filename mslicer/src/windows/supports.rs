use egui::{Context, Ui};
use slicer::supports::line::LineSupportGenerator;

use crate::{app::App, ui::components::dragger};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    if ui.button("Generate").clicked() {
        app.state.line_support_debug = Vec::new();
        let generator = LineSupportGenerator::new(
            &app.state.line_support_config,
            app.slice_config.platform_size,
        );

        for mesh in app.meshes.read().iter() {
            let supports = generator.generate_line_supports(&mesh.mesh);
            app.state.line_support_debug.extend(supports);
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
