use egui::{Context, Ui};
use slicer::supports::line::generate_line_supports;

use crate::{app::App, ui::components::dragger};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    if ui.button("Generate").clicked() {
        app.state.line_support_debug = Vec::new();
        for mesh in app.meshes.read().iter() {
            let supports = generate_line_supports(&mesh.mesh, &app.state.line_support_config);
            app.state.line_support_debug.extend(supports);
        }
    }

    ui.label("Support Config");

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
}
