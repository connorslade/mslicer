use common::misc::subscript_number;
use const_format::concatcp;
use egui::{Context, Grid, Id, Ui};
use egui_phosphor::regular::{
    ARROW_LINE_DOWN, CIRCLES_THREE, COPY, DICE_THREE, EYE, EYE_SLASH, LINK_BREAK, LINK_SIMPLE,
    TRASH, TRIANGLE, WARNING,
};
use nalgebra::Vector3;

use crate::{
    app::{App, model::MeshWarnings},
    ui::components::{vec3_dragger, vec3_dragger_proportional},
};

const WARN_NON_MANIFOLD: &str = "This mesh is non-manifold, it may produce unexpected results when sliced.\nConsider running it through a mesh repair tool.";
const WARN_OUT_OF_BOUNDS: &str = "This mesh extends beyond the printer volume and will be cut off.";

enum Action {
    None,
    Remove(usize),
    Duplicate(usize),
}

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let mut meshes = app.models.write();
    let mut action = Action::None;

    if meshes.is_empty() {
        ui.vertical_centered(|ui| {
            ui.label("No models loaded yet.");
        });
        return;
    }

    let width = ui.available_width();

    for (i, model) in meshes.iter_mut().enumerate() {
        let id = Id::new(format!("model_show_{}", model.id));
        let open = ui.data_mut(|map| *map.get_temp_mut_or_insert_with(id, || false));

        ui.horizontal(|ui| {
            if ui.button(if open { "⏷" } else { "⏵" }).clicked() {
                ui.data_mut(|map| map.insert_temp(id, !open));
            }
            model.hidden ^= ui
                .button(if model.hidden { EYE_SLASH } else { EYE })
                .on_hover_text(if model.hidden { "Show" } else { "Hide" })
                .clicked();

            if !model.warnings.is_empty() {
                let count = model.warnings.bits().count_ones();
                let mut warn = ui.label(format!("{WARNING}{}", subscript_number(count)));
                for warning in model.warnings.iter() {
                    let desc = match warning {
                        MeshWarnings::NonManifold => WARN_NON_MANIFOLD,
                        MeshWarnings::OutOfBounds => WARN_OUT_OF_BOUNDS,
                        _ => unreachable!(),
                    };
                    warn = warn.on_hover_text(desc);
                }
            }

            if open {
                ui.text_edit_singleline(&mut model.name);
            } else {
                ui.label(&model.name);
            }
        });

        if open {
            Grid::new(format!("model_{}", model.id))
                .num_columns(2)
                .with_row_color(|row, style| (row % 2 == 0).then_some(style.visuals.faint_bg_color))
                .show(ui, |ui| {
                    ui.label("Info");
                    ui.horizontal(|ui| {
                        ui.label(TRIANGLE);
                        ui.monospace(model.mesh.face_count().to_string());

                        ui.separator();

                        ui.label(CIRCLES_THREE);
                        ui.monospace(model.mesh.vertex_count().to_string());
                    });
                    ui.end_row();

                    ui.label("Actions");
                    ui.vertical(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.button(concatcp!(TRASH, " Delete"))
                                .clicked()
                                .then(|| action = Action::Remove(i));
                            ui.button(concatcp!(COPY, " Duplicate"))
                                .clicked()
                                .then(|| action = Action::Duplicate(i));
                            ui.button(concatcp!(ARROW_LINE_DOWN, " Align to Bed"))
                                .clicked()
                                .then(|| {
                                    model.align_to_bed();
                                    model.update_oob(&app.slice_config);
                                });
                        });
                    });
                    ui.end_row();

                    ui.horizontal(|ui| {
                        ui.label("Position");
                        ui.add_space(20.0);
                    });
                    ui.horizontal(|ui| {
                        let mut position = model.mesh.position();
                        vec3_dragger(ui, position.as_mut(), |x| x);
                        (model.mesh.position() != position)
                            .then(|| model.set_position(app, position));
                        ui.add_space(width);
                    });
                    ui.end_row();

                    ui.label("Scale");

                    ui.horizontal(|ui| {
                        let mut scale = model.mesh.scale();
                        if model.locked_scale {
                            vec3_dragger_proportional(ui, scale.as_mut(), |x| {
                                x.speed(0.01).range(0.001..=f32::MAX)
                            });
                        } else {
                            vec3_dragger(ui, scale.as_mut(), |x| {
                                x.speed(0.01).range(0.001..=f32::MAX)
                            });
                        }
                        (model.mesh.scale() != scale).then(|| model.set_scale(app, scale));

                        model.locked_scale ^= ui
                            .button(if model.locked_scale {
                                LINK_SIMPLE
                            } else {
                                LINK_BREAK
                            })
                            .clicked();
                    });
                    ui.end_row();

                    ui.label("Rotation");
                    let mut rotation = rad_to_deg(model.mesh.rotation());
                    let original_rotation = rotation;
                    vec3_dragger(ui, rotation.as_mut(), |x| x.suffix("°"));
                    (original_rotation != rotation)
                        .then(|| model.set_rotation(app, deg_to_rad(rotation)));
                    ui.end_row();

                    ui.label("Color");
                    ui.horizontal(|ui| {
                        ui.color_edit_button_rgb(model.color.as_slice_mut());
                        ui.button(concatcp!(DICE_THREE, " Random"))
                            .clicked()
                            .then(|| model.randomize_color());
                    });

                    ui.label("Name");
                    ui.text_edit_singleline(&mut model.name);
                    ui.end_row();
                });
        }
    }

    match action {
        Action::Remove(i) => {
            let id = meshes.remove(i).id;
            let id = Id::new(format!("model_show_{id}",));
            ui.data_mut(|map| map.remove::<bool>(id));
        }
        Action::Duplicate(i) => {
            let mesh = meshes[i].clone();
            meshes.push(mesh);
        }
        Action::None => {}
    }
}

fn rad_to_deg(pos: Vector3<f32>) -> Vector3<f32> {
    pos.map(|x| x.to_degrees())
}

fn deg_to_rad(pos: Vector3<f32>) -> Vector3<f32> {
    pos.map(|x| x.to_radians())
}
