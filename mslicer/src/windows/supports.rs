use egui::{CollapsingHeader, Context, Ui};
use nalgebra::Vector3;
use slicer::{
    builder::MeshBuilder,
    supports::{SupportGenerator, route_support},
};

use crate::{
    app::{
        App,
        project::model::Model,
        task::{BuildAccelerationStructures, MeshManifold},
    },
    ui::components::dragger,
};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.label("This feature is still very early in development.");

    ui.add_space(8.0);
    ui.heading("Overhang Detection");

    let overhang = &mut app.config.overhang_visualization;
    ui.checkbox(&mut overhang.0, "Visualize Overhanging Faces");
    dragger(ui, "Overhang Angle", &mut overhang.1, |x| x.speed(0.1));

    ui.add_space(8.0);
    ui.menu_button("Detect Overhanging Points", |ui| {
        ui.style_mut().visuals.button_frame = false;
        for idx in 0..app.project.models.len() {
            let model = &mut app.project.models[idx];
            if ui.button(&model.name).clicked() {
                model.find_overhangs();
            }
        }
    });

    ui.add_space(8.0);
    ui.heading("Manual Supports");
    ui.label("Unfinished!");

    ui.checkbox(&mut app.state.support_placement, "Support Placement");

    ui.add_space(8.0);
    ui.heading("Automatic Supports");

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.menu_button("Generate", |ui| {
            ui.style_mut().visuals.button_frame = false;

            for idx in 0..app.project.models.len() {
                if ui.button(&app.project.models[idx].name).clicked() {
                    app.state.line_support_debug = generate_support(app, idx);
                }
            }
        });

        if ui.button("Generate All").clicked() {
            app.state.line_support_debug = Vec::new();
            for i in 0..app.project.models.len() {
                let debug = generate_support(app, i);
                app.state.line_support_debug.extend_from_slice(&debug);
            }
        }
    });

    ui.add_space(8.0);
    let support = &mut app.state.support_config;

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
            ("Tip Radius", &mut support.tip_radius),
            ("Raft Height", &mut support.raft_height),
            ("Raft Offset", &mut support.raft_offset),
        ] {
            dragger(ui, name, value, |x| x.speed(0.1));
        }

        dragger(ui, "Support Precision", &mut support.precision, |x| x);
    });

    (app.state.support_placement).then(|| manual_support_placement(app));
}

fn generate_support(app: &mut App, model: usize) -> Vec<[Vector3<f32>; 2]> {
    let model = &app.project.models[model];
    let half_edge = model.half_edge.as_ref().unwrap();
    let bvh = model.bvh.as_ref().unwrap();

    let generator = SupportGenerator::new(
        &app.state.support_config,
        app.project.slice_config.platform_size.map(|x| x.convert()),
    );
    let (supports, debug) = generator.generate_supports(&model.mesh, half_edge, bvh);

    let mut model = Model::from_mesh(supports)
        .with_name(format!("Supports {}", model.name))
        .with_random_color();
    model.update_oob(&app.project.slice_config.platform_size);
    app.tasks.add(MeshManifold::new(&model));
    app.tasks.add(BuildAccelerationStructures::new(&model));
    app.project.models.push(model);

    debug
}

fn manual_support_placement(app: &mut App) {
    let workspace = &app.state.workspace;
    if workspace.is_moving {
        return;
    }

    let Some((pos, dir)) = app.hovered_ray() else {
        return;
    };

    let mut builder = MeshBuilder::new();
    for model in app.project.models.iter() {
        let Some(bvh) = model.bvh.as_ref() else {
            continue;
        };

        let Some(intersection) = bvh.intersect_ray(&model.mesh, pos, dir) else {
            continue;
        };

        let normal = (model.mesh).transform_normal(&model.mesh.normal(intersection.face));
        let start = intersection.position + normal * 0.1;

        if let Some(lines) = route_support(&model.mesh, bvh, start) {
            let (r, p) = (1.0, 100);
            builder.add_cylinder((intersection.position, start), (0.2, r), p);
            builder.add_cylinder((lines[0], lines[1]), (r, r), p);
            builder.add_cylinder((lines[1], lines[2]), (r, r), p);

            builder.add_sphere(intersection.position, 0.2, p);
            builder.add_sphere(lines[0], r, p);
            builder.add_sphere(lines[1], r, p);
        }
    }

    app.state.support_preview = (!builder.is_empty()).then(|| builder.build());
}
