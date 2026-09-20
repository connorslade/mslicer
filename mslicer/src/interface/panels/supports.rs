use const_format::concatcp;
use egui::{Button, Context, Ui, Widget};
use egui_phosphor::regular::{SELECTION_INVERSE, SPARKLE, TRASH, USER_SWITCH};
use slicer::builder::MeshBuilder;
use tools::supports::{SupportGenerator, route_support};

use crate::{core::App, interface::components::dragger};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let support_mode = &mut app.state.support_mode;
    ui.horizontal(|ui| {
        *support_mode ^= Button::selectable(!*support_mode, "Place").ui(ui).clicked();
        *support_mode ^= Button::selectable(*support_mode, "Edit").ui(ui).clicked();
    });
    ui.separator();

    if *support_mode {
        // edit
        let selected = &mut app.state.selected_supports;
        let selected_count = selected.count();

        if selected_count == 0 {
            ui.label("No supports selected.");
        } else {
            let models = selected.model_count();
            ui.label(format!(
                "{selected_count} support{} selected across {models} model{}.",
                ["", "s"][(selected_count != 1) as usize],
                ["", "s"][(models != 1) as usize],
            ));

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button(concatcp!(TRASH, " Delete")).clicked() {
                    for support in selected.iter() {
                        let model = app.project.model(support.model).unwrap();
                        model.supports.remove(support.support);
                        model.supports.invalidate_cache();
                    }
                    selected.clear();
                }

                ui.button(concatcp!(SELECTION_INVERSE, " Deselect"))
                    .clicked()
                    .then(|| selected.clear());
            });
        }
    } else {
        // place
        ui.horizontal(|ui| {
            let placement = &mut app.state.support_placement;
            *placement ^= Button::selectable(*placement, concatcp!(USER_SWITCH, " Manual"))
                .ui(ui)
                .clicked();
            app.state.support_placement &= !ui
                .menu_button(concatcp!(SPARKLE, " Auto"), |ui| {
                    ui.set_width(200.0);

                    for idx in 0..app.project.models.len() {
                        if ui.button(&app.project.models[idx].name).clicked() {
                            generate_support(app, idx);
                        }
                    }
                })
                .response
                .clicked();
        });

        ui.add_space(8.0);
        let support = &mut app.state.support_config;

        ui.collapsing("Support Placement", |ui| {
            dragger(ui, "Max Angle", &mut support.max_angle, |x| x.speed(0.01));
            dragger(
                ui,
                "Face Support Spacing",
                &mut support.face_support_spacing,
                |x| x,
            );
            dragger(ui, "Edge Angle Delta", &mut support.edge_angle_delta, |x| x);
            dragger(
                ui,
                "Edge Support Spacing",
                &mut support.edge_support_spacing,
                |x| x,
            );
            dragger(
                ui,
                "Minimum Support Spacing",
                &mut support.min_spacing,
                |x| x,
            );
        });

        ui.collapsing("Support Generation", |ui| {
            for (name, value) in [
                ("Support Radius", &mut support.support_radius),
                ("Tip Length", &mut support.tip_length),
                ("Tip Radius", &mut support.tip_radius),
                ("Raft Height", &mut support.raft_height),
                ("Raft Offset", &mut support.raft_offset),
            ] {
                dragger(ui, name, value, |x| x.speed(0.1));
            }

            dragger(ui, "Support Precision", &mut support.precision, |x| x);
        });
    }

    (app.state.support_placement).then(|| manual_support_placement(app, false));
}

fn generate_support(app: &mut App, model: usize) {
    let model = &mut app.project.models[model];
    let half_edge = model.half_edge.as_ref().unwrap();
    let bvh = model.bvh.as_ref().unwrap();

    let support_config = &app.state.support_config;
    let platform_size = app.project.slice_config.platform_size.map(|x| x.convert());

    let generator = SupportGenerator::new(support_config, platform_size);
    let supports = generator.generate_supports(&model.mesh, half_edge, bvh);
    model.supports.replace_auto(support_config, supports);
}

pub fn manual_support_placement(app: &mut App, clicked: bool) {
    let workspace = &app.state.workspace;
    if workspace.is_moving {
        return;
    }

    let Some((pos, dir)) = app.hovered_ray() else {
        return;
    };

    let mut builder = MeshBuilder::new();
    for model in app.project.models.iter_mut() {
        let Some(bvh) = model.bvh.as_ref() else {
            continue;
        };

        let Some(intersection) = bvh.intersect_ray(&model.mesh, pos, dir) else {
            continue;
        };

        let config = &app.state.support_config;
        let normal = (model.mesh).transform_normal(&model.mesh.normal(intersection.face));
        let intersection = model.mesh.transform(&intersection.position);
        let start = intersection + normal * config.tip_length;

        if let Some(middle) = route_support(&model.mesh, bvh, start) {
            let (r, p) = (1.0, 100);
            builder.add_cylinder((intersection, start), (config.tip_radius, r), p);
            builder.add_cylinder((start, middle), (r, r), p);
            builder.add_cylinder((middle, middle.xy().push(0.0)), (r, r), p);

            builder.add_sphere(intersection, 0.2, p);
            builder.add_sphere(start, r, p);
            builder.add_sphere(middle, r, p);

            if clicked {
                let support = [intersection, start, middle];
                model.supports.add_manual(config, support);
            }
        }
    }

    app.state.support_preview = (!builder.is_empty()).then(|| builder.build());
}

// NO DRUGS
// remember that.
