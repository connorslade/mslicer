use std::f32;

use const_format::concatcp;
use egui::{
    Color32, Context, DragAndDrop, Frame, ScrollArea, Sense, Stroke, StrokeKind, TopBottomPanel,
    Ui, Vec2,
};
use egui_phosphor::regular::{COPY, CURSOR_TEXT, EYE, EYE_SLASH, FOLDER_DASHED, SELECTION, TRASH};

use crate::{
    app::App,
    project::{Collection, CollectionId},
    windows::models::{
        collection::{collection, collection_entry},
        model::model_properties,
    },
};

mod collection;
mod model;

enum Action {
    None,
    Remove(usize),
    Duplicate(usize),
}

struct DraggedModel {
    index: usize,
    offset: Vec2,
}

pub fn ui(app: &mut App, ui: &mut Ui, ctx: &Context) {
    if app.project.models.is_empty() {
        ui.vertical_centered(|ui| {
            ui.label("No models loaded yet.");
        });
        return;
    }

    let pannel = TopBottomPanel::bottom("model_props")
        .resizable(false)
        .frame(Frame::new().inner_margin(2.0))
        .default_height(f32::MAX);

    if let Some(id) = app.state.selected.single_model()
        && let Some(model) = app.project.models.iter_mut().position(|x| x.id == id)
    {
        let mut action = Action::None;
        pannel.show_inside(ui, |ui| {
            ui.add_space(4.0);
            model_properties(app, ui, ctx, &mut action, model)
        });

        match action {
            Action::Remove(i) => {
                app.project.models.remove(i);
            }
            Action::Duplicate(i) => {
                let model = app.project.models[i].clone();
                app.project.models.push(model);
            }
            Action::None => {}
        }
    } else if app.state.selected.has_models() {
        pannel.show_inside(ui, |ui| {
            ui.add_space(4.0);
            selection_properties(app, ui);
        });
    } else if let Some(id) = app.state.selected.single_collection() {
        pannel.show_inside(ui, |ui| {
            ui.add_space(4.0);
            collection_properties(app, ui, id);
        });
    }

    ScrollArea::vertical().show(ui, |ui| {
        let mut n = 0;

        let mut rects = Vec::new();
        let mut header_rects = Vec::new();

        for i in 0..app.project.collections.len() {
            let header_rect = collection_entry(app, ui, i, &mut n, false);

            let group = &app.project.collections[i];
            header_rects.push((group.id, header_rect));
            n += 1;

            if !group.collapsed {
                collection(app, ctx, ui, Some(group.id), &mut n, &mut rects);
            }
        }
        collection(app, ctx, ui, None, &mut n, &mut rects);

        if let Some(pointer) = ctx.pointer_interact_pos()
            && DragAndDrop::has_any_payload(ctx)
        {
            let header_target = (header_rects.iter())
                .find(|(_, rect)| rect.contains(pointer))
                .map(|&(id, rect)| (id, rect));

            let stroke = Stroke::new(1.0, Color32::WHITE);
            if let Some((coll_id, header_rect)) = header_target {
                ui.painter()
                    .rect_stroke(header_rect, 2.0, stroke, StrokeKind::Outside);

                if ctx.input(|i| i.pointer.any_released())
                    && let Some(dragged) = DragAndDrop::payload::<DraggedModel>(ctx)
                {
                    let model = app.project.models.remove(dragged.index);
                    let insert_pos = (app.project.models.iter())
                        .rposition(|m| m.collection == Some(coll_id))
                        .map(|i| i + 1)
                        .unwrap_or(app.project.models.len());
                    app.project.models.insert(insert_pos, model);
                    app.project.models[insert_pos].collection = Some(coll_id);
                }
            } else {
                let insert_slot = (rects.iter())
                    .position(|(_, _, rect)| pointer.y < rect.center().y)
                    .unwrap_or(rects.len());

                let line_y = if insert_slot < rects.len() {
                    rects[insert_slot].2.min.y
                } else {
                    rects.last().map(|(_, _, r)| r.max.y).unwrap_or_default()
                };
                ui.painter().hline(ui.max_rect().x_range(), line_y, stroke);

                if ctx.input(|i| i.pointer.any_released())
                    && let Some(dragged) = DragAndDrop::payload::<DraggedModel>(ctx)
                {
                    let target_collection = if insert_slot < rects.len() {
                        rects[insert_slot].1
                    } else {
                        rects.last().and_then(|&(_, c, _)| c)
                    };

                    let to = if insert_slot < rects.len() {
                        rects[insert_slot].0
                    } else {
                        rects.last().map(|&(i, _, _)| i + 1).unwrap_or(0)
                    };

                    let to = to.saturating_sub((dragged.index < to) as usize);
                    let model = app.project.models.remove(dragged.index);
                    app.project.models.insert(to, model);
                    app.project.models[to].collection = target_collection;
                }
            }
        }

        let fill = ui.available_rect_before_wrap();
        let resp = ui.interact(fill, ui.id().with("deselect"), Sense::click());
        resp.clicked().then(|| app.state.selected.clear());
    });
}

fn selection_properties(app: &mut App, ui: &mut Ui) {
    ui.horizontal_wrapped(|ui| {
        if ui.button(concatcp!(FOLDER_DASHED, " Collect")).clicked() {
            let collection = Collection::new_unnamed();
            for id in app.state.selected.selected_models() {
                if let Some(model) = app.project.models.iter_mut().find(|x| x.id == id) {
                    model.collection = Some(collection.id);
                }
            }
            app.project.collections.push(collection);
        }

        if ui.button(concatcp!(COPY, " Duplicate")).clicked() {
            for id in app.state.selected.selected_models() {
                if let Some(model) = app.project.models.iter().find(|x| x.id == id) {
                    app.project.models.push(model.clone());
                }
            }
        }
    });
}

fn collection_properties(app: &mut App, ui: &mut Ui, id: CollectionId) {
    ui.horizontal_wrapped(|ui| {
        if ui.button(concatcp!(CURSOR_TEXT, " Rename")).clicked()
            && let Some(collection) = app.project.collection(id)
        {
            collection.rename = crate::project::RenameState::Starting;
        }

        if ui.button(concatcp!(TRASH, " Delete")).clicked() {
            app.state.selected.clear();
            app.project.collections.retain(|x| x.id != id);
            app.project.models.iter_mut().for_each(|m| {
                (m.collection == Some(id)).then(|| m.collection = None);
            });
        }

        let hidden = (app.project.models.iter())
            .find(|x| x.collection == Some(id))
            .map(|x| x.hidden)
            .unwrap_or_default();
        let text = [
            concatcp!(EYE_SLASH, " Hide All"),
            concatcp!(EYE, " Show All"),
        ][hidden as usize];
        if ui.button(text).clicked() {
            (app.project.models.iter_mut())
                .filter(|x| x.collection == Some(id))
                .for_each(|m| m.hidden = !hidden);
        }

        if ui.button(concatcp!(SELECTION, " Select Models")).clicked() {
            (app.project.models.iter())
                .filter(|x| x.collection == Some(id))
                .for_each(|x| app.state.selected.select_model(x.id));
        }
    });
}
