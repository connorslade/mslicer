use egui::{CollapsingHeader, Context, Ui};
use nalgebra::Vector3;
use slicer::supports::{line::LineSupportGenerator, route_support};

use crate::{app::App, render::model::Model, ui::components::dragger};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.label("This feature is still very early in development.");
    ui.add_space(8.0);

    ui.heading("Overhang Detection");

    let min_angle = &mut app.state.line_support_config.min_angle;
    ui.checkbox(&mut true, "Visualize Overhanging Faces");
    dragger(ui, "Overhang Angle", min_angle, |x| x.speed(0.1));

    ui.add_space(8.0);
    let mut models = app.models.write();
    ui.menu_button("Detect Overhanging Points", |ui| {
        ui.style_mut().visuals.button_frame = false;
        for idx in 0..models.len() {
            let model = &mut models[idx];
            if ui.button(&model.name).clicked() {
                model.find_overhangs();

                let verts = model.mesh.vertices();
                let lines = &mut app.state.line_support_debug;
                let mut line = |a, b| lines.push([a, b - a]);

                for overhang in model.overhangs.as_ref().unwrap() {
                    let point = model.mesh.transform(&verts[*overhang as usize]);

                    let start = point - Vector3::z() * 0.1;
                    if let Some(lines) = route_support(&model.mesh, &model.bvh, start) {
                        line(point, start);
                        line(lines[0], lines[1]);
                        line(lines[1], lines[2]);
                    }
                }
            }
        }
    });

    ui.add_space(8.0);
    ui.heading("Automatic Supports");

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.menu_button("Generate", |ui| {
            ui.style_mut().visuals.button_frame = false;

            for idx in 0..models.len() {
                let mesh = &models[idx];
                if ui.button(&mesh.name).clicked() {
                    let generator = LineSupportGenerator::new(
                        &app.state.line_support_config,
                        app.slice_config.platform_size,
                    );

                    app.state.line_support_debug = generate_support(&mut models, idx, &generator);
                }
            }
        });

        if ui.button("Generate All").clicked() {
            app.state.line_support_debug = Vec::new();
            let generator = LineSupportGenerator::new(
                &app.state.line_support_config,
                app.slice_config.platform_size,
            );

            for i in 0..models.len() {
                let debug = generate_support(&mut models, i, &generator);
                app.state.line_support_debug.extend_from_slice(&debug);
            }
        }
    });
    drop(models);

    ui.add_space(8.0);
    let support = &mut app.state.line_support_config;

    CollapsingHeader::new("Overhang Detection").show(ui, |ui| {
        dragger(ui, "Min Angle", &mut support.min_angle, |x| x.speed(0.01));
        dragger(
            ui,
            "Face Support Spacing",
            &mut support.face_support_spacing,
            |x| x,
        );
    });

    CollapsingHeader::new("Support Generation").show(ui, |ui| {
        for (name, value) in [
            ("Support Radius", &mut support.support_radius),
            ("Arm Height", &mut support.arm_height),
            ("Base Radius", &mut support.base_radius),
            ("Base Height", &mut support.base_height),
        ] {
            dragger(ui, name, value, |x| x.speed(0.1));
        }

        dragger(
            ui,
            "Support Precision",
            &mut support.support_precision,
            |x| x,
        );
    });
}

fn generate_support(
    meshes: &mut Vec<Model>,
    idx: usize,
    support: &LineSupportGenerator,
) -> Vec<[Vector3<f32>; 2]> {
    let mesh = &meshes[idx];

    let (supports, debug) = support.generate_line_supports(&mesh.mesh);
    let mesh = Model::from_mesh(supports)
        .with_name(format!("Supports {}", mesh.name))
        .with_random_color();

    meshes.push(mesh);
    debug
}
