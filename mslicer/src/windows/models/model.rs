use std::f32;

use common::{
    misc::{separate_thousands, subscript_number},
    units::Centimeter,
};
use const_format::concatcp;
use egui::{
    CollapsingHeader, Color32, Context, DragValue, Label, Popup, Response, Sense, Ui, UiBuilder,
    Widget, vec2,
};
use egui_phosphor::regular::{
    ARROW_LINE_DOWN, COPY, CURSOR_TEXT, DICE_THREE, EYE, EYE_SLASH, FOLDER_DASHED, LINK_BREAK,
    LINK_SIMPLE, TRASH, WARNING,
};
use nalgebra::Vector3;

use crate::{
    app::{App, history::ModelAction},
    project::{
        Collection,
        model::{MeshWarnings, RenameState},
    },
    ui::components::{
        being_edited, grid, history_tracked_model, vec3_dragger, vec3_dragger_proportional,
    },
    windows::models::Action,
};

const WARN_NON_MANIFOLD: &str = "This mesh is non-manifold, it may produce unexpected results when sliced.\nConsider running it through a mesh repair tool.";
const WARN_OUT_OF_BOUNDS: &str = "This mesh extends beyond the printer volume and will be cut off.";

pub fn model_entry(app: &mut App, ui: &mut Ui, idx: usize, dragged: bool) -> Response {
    let model = &mut app.project.models[idx];
    let id = model.id;

    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::click_and_drag());

    let selected = app.state.selected.contains_model(id);
    let color = if selected && !dragged {
        ui.visuals().selection.bg_fill
    } else if response.hovered() || dragged {
        ui.visuals().code_bg_color
    } else if idx % 2 == 1 {
        ui.visuals().faint_bg_color
    } else {
        ui.style().noninteractive().bg_fill
    };

    let rect_margin = rect.expand2(vec2(2.0, ui.spacing().item_spacing.y / 2.0));
    ui.painter().rect_filled(rect_margin, 2.0, color);

    if response.clicked() {
        app.state
            .selected
            .model_clicked(id, ui.input(|x| x.modifiers.shift));
    }

    let indent = !dragged && model.collection.is_some();
    let ui_builder =
        UiBuilder::new().max_rect(rect.with_min_x(rect.min.x + [0.0, 12.0][indent as usize]));
    ui.scope_builder(ui_builder, |ui| {
        ui.horizontal(|ui| {
            ui.visuals_mut().override_text_color = selected.then_some(Color32::WHITE);
            ui.visuals_mut().button_frame = false;

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
                Label::new(&model.name).selectable(false).ui(ui);
            }

            ui.take_available_width();
        });
    });

    response
}

pub fn model_properties(app: &mut App, ui: &mut Ui, ctx: &Context, action: &mut Action, i: usize) {
    let model = &mut app.project.models[i];

    let platform = &app.project.slice_config.platform_size;
    let id = model.id;

    ui.horizontal_wrapped(|ui| {
        (ui.button(concatcp!(CURSOR_TEXT, " Rename")).clicked())
            .then(|| model.ui.rename = RenameState::Starting);
        (ui.button(concatcp!(TRASH, " Delete")).clicked()).then(|| *action = Action::Remove(i));
        (ui.button(concatcp!(COPY, " Duplicate")).clicked())
            .then(|| *action = Action::Duplicate(i));
        ui.button(concatcp!(FOLDER_DASHED, " Collect"))
            .clicked()
            .then(|| {
                let collection = Collection::new("Collection".into());
                model.collection = Some(collection.id);
                app.project.collections.push(collection);
            });
        ui.button(concatcp!(ARROW_LINE_DOWN, " Align to Bed"))
            .clicked()
            .then(|| {
                let old_pos = model.mesh.position();
                app.history.track_model(id, ModelAction::Position(old_pos));

                model.align_to_bed();
                model.update_oob(platform);
            });
    });

    CollapsingHeader::new("Transform")
        .default_open(true)
        .show(ui, |ui| {
            grid("model_props_grid").show(ui, |ui| {
                ui.label("Position");
                ui.horizontal(|ui| {
                    let mut position = model.mesh.position();
                    let editing = vec3_dragger(ui, position.as_mut(), |x| x);
                    history_tracked_model(
                        (editing, ui, &mut app.history),
                        (id, || ModelAction::Position(model.mesh.position())),
                    );
                    (model.mesh.position() != position)
                        .then(|| model.set_position(platform, position));
                    ui.take_available_width();
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

    ui.collapsing("Miscellaneous", |ui| {
        grid("model_props_grid").show(ui, |ui| {
            ui.label("Faces");
            ui.horizontal(|ui| {
                ui.label(separate_thousands(model.mesh.face_count()));
                ui.take_available_width();
            });
            ui.end_row();

            ui.label("Volume");
            let volume = model.volume().convert::<Centimeter>().raw();
            ui.label(format!("{volume:.2} cm³"));
            ui.end_row();

            ui.label("Exposure");
            let mut value = model.exposure as f32 / 2.55;
            let editing = being_edited(
                &DragValue::new(&mut value)
                    .range(0.0..=100.0)
                    .max_decimals(0)
                    .suffix("%")
                    .ui(ui),
            );

            history_tracked_model(
                (editing, ui, &mut app.history),
                (id, || ModelAction::RelativeExposure(model.exposure)),
            );
            editing.then(|| model.exposure = (value * 2.55).round() as u8);

            ui.end_row();
        });
    });
}

fn rad_to_deg(pos: Vector3<f32>) -> Vector3<f32> {
    pos.map(|x| x.to_degrees())
}

fn deg_to_rad(pos: Vector3<f32>) -> Vector3<f32> {
    pos.map(|x| x.to_radians())
}
