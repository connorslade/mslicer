use std::mem;

use common::misc::subscript_number;
use const_format::concatcp;
use egui::{Align, Context, Grid, Layout, Popup, Ui, collapsing_header::CollapsingState};
use egui_phosphor::regular::{
    ARROW_LINE_DOWN, COPY, CURSOR_TEXT, DICE_THREE, DOTS_THREE_CIRCLE, EYE, EYE_SLASH, LINK_BREAK,
    LINK_SIMPLE, TRASH, WARNING,
};
use nalgebra::Vector3;

use crate::{
    app::{
        App,
        history::ModelAction,
        project::model::{MeshWarnings, RenameState},
    },
    ui::components::{
        being_edited, history_tracked_model, vec3_dragger, vec3_dragger_proportional,
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

    let platform = &app.project.slice_config.platform_size;
    for (i, model) in app.project.models.iter_mut().enumerate() {
        let id = model.id;
        let collapse_id = ui.id().with(id);

        let mut collapsing = CollapsingState::load_with_default_open(ui.ctx(), collapse_id, false);
        mem::take(&mut model.ui.toggle).then(|| collapsing.toggle(ui));

        collapsing
            .show_header(ui, |ui| {
                ui.visuals_mut().button_frame = false;

                if !matches!(model.ui.rename, RenameState::None) {
                    let text_edit = ui.text_edit_singleline(&mut model.name);
                    if matches!(model.ui.rename, RenameState::Starting) {
                        text_edit.request_focus();
                        model.ui.rename = RenameState::Editing;
                    }

                    let editing = being_edited(&text_edit);
                    (!editing).then(|| model.ui.rename = RenameState::None);

                    history_tracked_model(
                        (editing, ui, &mut app.history),
                        (id, || ModelAction::Name(model.name.clone())),
                    )
                } else {
                    model.ui.toggle ^= ui.button(&model.name).clicked();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.menu_button(DOTS_THREE_CIRCLE, |ui| {
                        (ui.button(concatcp!(CURSOR_TEXT, " Rename")).clicked())
                            .then(|| model.ui.rename = RenameState::Starting);
                        (ui.button(concatcp!(TRASH, " Delete")).clicked())
                            .then(|| action = Action::Remove(i));
                        (ui.button(concatcp!(COPY, " Duplicate")).clicked())
                            .then(|| action = Action::Duplicate(i));
                        ui.button(concatcp!(ARROW_LINE_DOWN, " Align to Bed"))
                            .clicked()
                            .then(|| {
                                let old_pos = model.mesh.position();
                                app.history.track_model(id, ModelAction::Position(old_pos));

                                model.align_to_bed();
                                model.update_oob(platform);
                            });
                    });

                    if ui
                        .button(if model.hidden { EYE_SLASH } else { EYE })
                        .on_hover_text(if model.hidden { "Show" } else { "Hide" })
                        .clicked()
                    {
                        app.history
                            .track_model(model.id, ModelAction::Hidden(model.hidden));
                        model.hidden ^= true;
                    }

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
                });
            })
            .body(|ui| {
                Grid::new(format!("model_{}", model.id))
                    .num_columns(2)
                    .with_row_color(|row, style| {
                        (row % 2 == 0).then_some(style.visuals.faint_bg_color)
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Position");
                            ui.add_space(20.0);
                        });
                        ui.horizontal(|ui| {
                            let mut position = model.mesh.position();
                            let editing = vec3_dragger(ui, position.as_mut(), |x| x);
                            history_tracked_model(
                                (editing, ui, &mut app.history),
                                (id, || ModelAction::Position(model.mesh.position())),
                            );
                            (model.mesh.position() != position)
                                .then(|| model.set_position(platform, position));
                            ui.add_space(ui.available_width());
                        });
                        ui.end_row();

                        ui.label("Scale");

                        ui.horizontal(|ui| {
                            let mut scale = model.mesh.scale();
                            let editing = if model.ui.locked_scale {
                                vec3_dragger_proportional(ui, scale.as_mut(), |x| {
                                    x.speed(0.01).range(0.001..=f32::MAX)
                                })
                            } else {
                                vec3_dragger(ui, scale.as_mut(), |x| {
                                    x.speed(0.01).range(0.001..=f32::MAX)
                                })
                            };
                            history_tracked_model(
                                (editing, ui, &mut app.history),
                                (id, || ModelAction::Scale(model.mesh.scale())),
                            );
                            (model.mesh.scale() != scale).then(|| model.set_scale(platform, scale));

                            model.ui.locked_scale ^= ui
                                .button([LINK_BREAK, LINK_SIMPLE][model.ui.locked_scale as usize])
                                .clicked();
                        });
                        ui.end_row();

                        ui.label("Rotation");
                        let mut rotation = rad_to_deg(model.mesh.rotation());
                        let editing = vec3_dragger(ui, rotation.as_mut(), |x| x.suffix("°"));
                        history_tracked_model(
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
                            history_tracked_model(
                                (editing, ui, &mut app.history),
                                (id, || ModelAction::Color(original_color)),
                            );

                            if ui.button(concatcp!(DICE_THREE, " Random")).clicked() {
                                app.history.track_model(id, ModelAction::Color(model.color));
                                model.randomize_color();
                            }
                        });
                    });
            });
    }

    match action {
        Action::Remove(i) => {
            app.project.models.remove(i).id;
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
