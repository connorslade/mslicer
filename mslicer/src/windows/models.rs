use common::misc::subscript_number;
use const_format::concatcp;
use egui::{Context, Grid, Id, Popup, Ui};
use egui_phosphor::regular::{
    ARROW_LINE_DOWN, CIRCLES_THREE, COPY, DICE_THREE, EYE, EYE_SLASH, LINK_BREAK, LINK_SIMPLE,
    TRASH, TRIANGLE, WARNING,
};
use nalgebra::Vector3;

use crate::{
    app::{App, history::ModelAction, project::model::MeshWarnings},
    ui::components::{
        being_edited, history_tracked_value, vec3_dragger, vec3_dragger_proportional,
    },
};

const WARN_NON_MANIFOLD: &str = "This mesh is non-manifold, it may produce unexpected results when sliced.\nConsider running it through a mesh repair tool.";
const WARN_OUT_OF_BOUNDS: &str = "This mesh extends beyond the printer volume and will be cut off.";

enum Action {
    None,
    Remove(usize),
    Duplicate(usize),
}

pub fn ui(app: &mut App, ui: &mut Ui, ctx: &Context) {
    let mut action = Action::None;

    if app.project.models.is_empty() {
        ui.vertical_centered(|ui| {
            ui.label("No models loaded yet.");
        });
        return;
    }

    let width = ui.available_width();
    let platform = &app.project.slice_config.platform_size;

    for (i, model) in app.project.models.iter_mut().enumerate() {
        let id = model.id;
        let data_id = Id::new("model_show").with(id);
        let open = ui.data_mut(|map| *map.get_temp_mut_or_insert_with(data_id, || false));

        ui.horizontal(|ui| {
            (ui.button(if open { "⏷" } else { "⏵" }).clicked())
                .then(|| ui.data_mut(|map| map.insert_temp(data_id, !open)));

            let toggle_history = ui
                .button(if model.hidden { EYE_SLASH } else { EYE })
                .on_hover_text(if model.hidden { "Show" } else { "Hide" })
                .clicked();
            if toggle_history {
                app.history
                    .track_model(model.id, ModelAction::Hidden(model.hidden))
            }
            model.hidden ^= toggle_history;

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
                let text_edit = ui.text_edit_singleline(&mut model.name);
                let editing = being_edited(&text_edit);
                history_tracked_value(
                    (editing, ui, &mut app.history),
                    (id, || ModelAction::Name(model.name.clone())),
                )
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
                            (ui.button(concatcp!(TRASH, " Delete")).clicked())
                                .then(|| action = Action::Remove(i));
                            (ui.button(concatcp!(COPY, " Duplicate")).clicked())
                                .then(|| action = Action::Duplicate(i));
                            ui.button(concatcp!(ARROW_LINE_DOWN, " Align to Bed"))
                                .clicked()
                                .then(|| {
                                    model.align_to_bed();
                                    model.update_oob(platform);
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
                        let editing = vec3_dragger(ui, position.as_mut(), |x| x);
                        history_tracked_value(
                            (editing, ui, &mut app.history),
                            (id, || ModelAction::Position(model.mesh.position())),
                        );
                        (model.mesh.position() != position)
                            .then(|| model.set_position(platform, position));
                        ui.add_space(width);
                    });
                    ui.end_row();

                    ui.label("Scale");

                    ui.horizontal(|ui| {
                        let mut scale = model.mesh.scale();
                        let editing = if model.locked_scale {
                            vec3_dragger_proportional(ui, scale.as_mut(), |x| {
                                x.speed(0.01).range(0.001..=f32::MAX)
                            })
                        } else {
                            vec3_dragger(ui, scale.as_mut(), |x| {
                                x.speed(0.01).range(0.001..=f32::MAX)
                            })
                        };
                        history_tracked_value(
                            (editing, ui, &mut app.history),
                            (id, || ModelAction::Scale(model.mesh.scale())),
                        );
                        (model.mesh.scale() != scale).then(|| model.set_scale(platform, scale));

                        model.locked_scale ^= ui
                            .button([LINK_BREAK, LINK_SIMPLE][model.locked_scale as usize])
                            .clicked();
                    });
                    ui.end_row();

                    ui.label("Rotation");
                    let mut rotation = rad_to_deg(model.mesh.rotation());
                    let editing = vec3_dragger(ui, rotation.as_mut(), |x| x.suffix("°"));
                    history_tracked_value(
                        (editing, ui, &mut app.history),
                        (id, || ModelAction::Rotation(model.mesh.rotation())),
                    );
                    (model.mesh.rotation() != rotation)
                        .then(|| model.set_rotation(platform, deg_to_rad(rotation)));
                    ui.end_row();

                    ui.label("Color");
                    ui.horizontal(|ui| {
                        let editing = Popup::is_id_open(ctx, ui.auto_id_with("popup"));
                        let original_color = model.color;
                        ui.color_edit_button_rgb(model.color.as_slice_mut());
                        history_tracked_value(
                            (editing, ui, &mut app.history),
                            (id, || ModelAction::Color(original_color)),
                        );

                        if ui.button(concatcp!(DICE_THREE, " Random")).clicked() {
                            app.history.track_model(id, ModelAction::Color(model.color));
                            model.randomize_color();
                        }
                    });
                });
        }
    }

    match action {
        Action::Remove(i) => {
            let id = app.project.models.remove(i).id;
            let id = Id::new(format!("model_show_{id}",));
            ui.data_mut(|map| map.remove::<bool>(id));
        }
        Action::Duplicate(i) => {
            let model = app.project.models[i].clone();
            app.project.models.push(model);
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
