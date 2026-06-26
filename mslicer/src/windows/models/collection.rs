use egui::{
    Color32, Context, DragAndDrop, Id, Label, LayerId, Order, Rect, Sense, Ui, UiBuilder, Widget,
    emath::TSTransform, vec2,
};
use egui_phosphor::regular::{FOLDER, FOLDER_OPEN};

use crate::{
    app::App,
    project::RenameState,
    ui::components::being_edited,
    windows::models::{DraggedModel, model::model_entry},
};

pub fn collection(
    app: &mut App,
    ctx: &Context,
    ui: &mut Ui,
    collection: Option<u32>,
    n: &mut usize,
    rects: &mut Vec<(usize, Option<u32>, Rect)>,
) {
    for j in 0..app.project.models.len() {
        let model = &app.project.models[j];
        if model.collection != collection {
            continue;
        }

        let id = model.id;
        let eid = Id::new("model").with(id);

        if ctx.is_being_dragged(eid) {
            let layer_id = LayerId::new(Order::Tooltip, eid);
            let response = ui.scope_builder(UiBuilder::new().layer_id(layer_id), |ui| {
                model_entry(app, ui, j, 0, true)
            });

            if let Some(pointer_pos) = ctx.pointer_interact_pos()
                && let Some(dragged) = DragAndDrop::payload::<DraggedModel>(ctx)
            {
                let delta = pointer_pos - response.response.rect.center() - dragged.offset;
                ctx.transform_layer_shapes(layer_id, TSTransform::from_translation(delta));
            }
        } else {
            let response = model_entry(app, ui, j, *n, false);
            rects.push((j, collection, response.rect));
            *n += 1;

            if response.drag_started() {
                ctx.set_dragged_id(eid);
                let offset = (ctx.pointer_interact_pos())
                    .map(|p| p - response.rect.center())
                    .unwrap_or_default();
                let payload = DraggedModel { index: j, offset };

                DragAndDrop::set_payload(ctx, payload);
            }
        }
    }
}

pub fn collection_entry(
    app: &mut App,
    ui: &mut Ui,
    group: usize,
    n: &mut usize,
    dragged: bool,
) -> Rect {
    let group = &mut app.project.collections[group];

    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::click_and_drag());

    let selected = app.state.selected.contains_collection(group.id);
    let color = if selected && !dragged {
        ui.visuals().selection.bg_fill
    } else if response.hovered() || dragged {
        ui.visuals().code_bg_color
    } else if *n % 2 == 1 {
        ui.visuals().faint_bg_color
    } else {
        ui.style().noninteractive().bg_fill
    };

    let rect_margin = rect.expand2(vec2(2.0, ui.spacing().item_spacing.y / 2.0));
    ui.painter().rect_filled(rect_margin, 2.0, color);

    if response.clicked() {
        group.collapsed ^= selected;
        app.state
            .selected
            .collection_clicked(group.id, ui.input(|x| x.modifiers.shift));
    }

    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.horizontal(|ui| {
            ui.visuals_mut().override_text_color = selected.then_some(Color32::WHITE);
            ui.visuals_mut().button_frame = false;

            if ui
                .button(if group.collapsed { FOLDER } else { FOLDER_OPEN })
                .clicked()
            {
                group.collapsed ^= true;
            }

            if matches!(group.rename, RenameState::None) {
                Label::new(&group.name).selectable(false).ui(ui);
            } else {
                let text_edit = ui.text_edit_singleline(&mut group.name);
                if matches!(group.rename, RenameState::Starting) {
                    text_edit.request_focus();
                    group.rename = RenameState::Editing;
                }

                let editing = being_edited(&text_edit);
                (!editing).then(|| group.rename = RenameState::None);
            }

            ui.take_available_width();
        });
    });

    rect
}
