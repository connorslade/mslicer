use std::f32::consts::PI;

use egui::{CollapsingHeader, Context, Ui, emath::OrderedFloat};
use nalgebra::{Vector2, Vector3};
use slicer::{
    builder::MeshBuilder,
    supports::{line::LineSupportGenerator, route_support},
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
                let bvh = model.bvh.as_ref().unwrap();

                let verts = model.mesh.vertices();
                let mut builder = MeshBuilder::new();

                let mut support_centers = Vec::new();
                for overhang in model.overhangs.as_ref().unwrap() {
                    let point = model.mesh.transform(&verts[*overhang as usize]);

                    let start = point - Vector3::z();
                    if let Some(lines) = route_support(&model.mesh, bvh, start) {
                        let (r, p) = (1.0, 10);
                        builder.add_cylinder((point, start), (0.2, r), p);
                        builder.add_cylinder((lines[0], lines[1]), (r, r), p);
                        builder.add_cylinder((lines[1], lines[2]), (r, r), p);

                        for i in 0..(p * 2) {
                            let angle = i as f32 / p as f32 * PI;
                            let normal = Vector2::new(angle.cos(), angle.sin());
                            support_centers.push(lines[2].xy() + normal * r);
                        }

                        builder.add_sphere(point, 0.2, p);
                        builder.add_sphere(lines[0], r, p);
                        builder.add_sphere(lines[1], r, p);
                    }
                }

                let hull = convex_hull(&support_centers);
                let idx = builder.next_idx();
                for i in 0..hull.len() {
                    let point = hull[i];
                    let next = hull[(i + 1) % hull.len()];
                    let prev = hull[(i + hull.len() - 1) % hull.len()];

                    let edge_1 = next - point;
                    let edge_2 = point - prev;
                    let offset = Vector2::new(edge_1.y, -edge_1.x).normalize()
                        + Vector2::new(edge_2.y, -edge_2.x).normalize();

                    builder.add_vertex(point.push(0.0));
                    builder.add_vertex((point - offset.normalize()).push(1.0));
                }

                let verts = builder.next_idx() - idx;
                for i in (0..verts).step_by(2) {
                    if i != 0 && i + 3 < verts {
                        builder.add_face([idx, idx + i, idx + i + 2]);
                        builder.add_face([idx + i + 3, idx + i + 1, idx + 1]);
                    }

                    builder.add_quad_flipped([
                        idx + i % verts,
                        idx + (i + 1) % verts,
                        idx + (i + 2) % verts,
                        idx + (i + 3) % verts,
                    ]);
                }

                if !builder.is_empty() {
                    let mesh = builder.build();
                    let mut model = Model::from_mesh(mesh)
                        .with_name("Supports".into())
                        .with_random_color();
                    model.update_oob(&app.project.slice_config.platform_size);
                    app.tasks.add(MeshManifold::new(&model));
                    app.tasks.add(BuildAccelerationStructures::new(&model));
                    app.project.models.push(model);
                }
            }
        }
    });

    ui.add_space(8.0);
    ui.heading("Manual Supports");

    ui.checkbox(&mut app.state.support_placement, "Support Placement");

    ui.add_space(8.0);
    ui.heading("Automatic Supports");

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.menu_button("Generate", |ui| {
            ui.style_mut().visuals.button_frame = false;

            for idx in 0..app.project.models.len() {
                let mesh = &app.project.models[idx];
                if ui.button(&mesh.name).clicked() {
                    let generator = LineSupportGenerator::new(
                        &app.state.line_support_config,
                        app.project.slice_config.platform_size.map(|x| x.convert()),
                    );

                    app.state.line_support_debug =
                        generate_support(&mut app.project.models, idx, &generator);
                }
            }
        });

        if ui.button("Generate All").clicked() {
            app.state.line_support_debug = Vec::new();
            let generator = LineSupportGenerator::new(
                &app.state.line_support_config,
                app.project.slice_config.platform_size.map(|x| x.convert()),
            );

            for i in 0..app.project.models.len() {
                let debug = generate_support(&mut app.project.models, i, &generator);
                app.state.line_support_debug.extend_from_slice(&debug);
            }
        }
    });

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

    (app.state.support_placement).then(|| manual_support_placement(app));
}

fn generate_support(
    meshes: &mut Vec<Model>,
    idx: usize,
    support: &LineSupportGenerator,
) -> Vec<[Vector3<f32>; 2]> {
    let mesh = &meshes[idx];

    let half_edge = mesh.half_edge.as_ref().unwrap();
    let (supports, debug) = support.generate_line_supports(&mesh.mesh, half_edge);
    let mesh = Model::from_mesh(supports)
        .with_name(format!("Supports {}", mesh.name))
        .with_random_color();

    meshes.push(mesh);
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

fn convex_hull(points: &[Vector2<f32>]) -> Vec<&Vector2<f32>> {
    let first = points.iter().min_by_key(|p| OrderedFloat(p.x)).unwrap();

    let mut hull = vec![first];
    let mut current = first;

    loop {
        let mut next = current;
        for point in points {
            if *point == *current {
                continue;
            }

            if *next == *current || is_left_turn(current, next, point) {
                next = point;
            }
        }

        if *next == *first {
            break;
        }

        hull.push(next);
        current = next;
    }

    hull
}

fn is_left_turn(a: &Vector2<f32>, b: &Vector2<f32>, c: &Vector2<f32>) -> bool {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    cross > 0.0 || (cross == 0.0 && (a - c).magnitude_squared() > (a - b).magnitude_squared())
}
