use std::f32;

use const_format::concatcp;
use egui::{
    Color32, Context, DragAndDrop, Frame, ScrollArea, Sense, Stroke, TopBottomPanel, Ui, Vec2,
};
use egui_phosphor::regular::FOLDER_DASHED;

use crate::{
    app::App,
    project::Collection,
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
    }

    ScrollArea::vertical().show(ui, |ui| {
        for i in 0..app.project.collections.len() {
            collection_entry(app, ui, i);
            let group = &app.project.collections[i];
            if !group.collapsed {
                collection(app, ctx, ui, Some(group.id));
            }
        }
        collection(app, ctx, ui, None);
    });

    if let Some(pointer) = ctx.pointer_interact_pos()
        && let Some(dragged) = DragAndDrop::payload::<DraggedModel>(ctx)
    {
        let rect = ui.max_rect();

        let stroke = Stroke::new(1.0, Color32::WHITE);
        let line = |y| ui.painter().hline(rect.x_range(), y, stroke);

        let entry_height = 18.0 + ui.style().spacing.item_spacing.y;
        let t = (pointer.y - rect.min.y) / entry_height + 0.5;
        let new_index = (t as usize).min(app.project.models.len());

        line(rect.min.y + new_index as f32 * entry_height);

        if ctx.input(|i| i.pointer.any_released()) {
            let insert_index = new_index - (dragged.index < new_index) as usize;
            let model = app.project.models.remove(dragged.index);
            app.project.models.insert(insert_index, model);
        }
    }

    let (_rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    (response.clicked()).then(|| app.state.selected.clear());
}

fn selection_properties(app: &mut App, ui: &mut Ui) {
    ui.horizontal_wrapped(|ui| {
        if ui.button(concatcp!(FOLDER_DASHED, " Collect")).clicked() {
            let collection = Collection::new("Collection".into());
            for id in app.state.selected.selected_models() {
                if let Some(model) = app.project.models.iter_mut().find(|x| x.id == id) {
                    model.collection = Some(collection.id);
                }
            }
            app.project.collections.push(collection);
        }
    });
}
