use std::{hash::Hash, mem};

use egui::{
    Align, Button, CollapsingHeader, Color32, FontId, Grid, Layout, OpenUrl, Response, RichText,
    Separator, Ui, vec2,
};
use egui_phosphor::regular::LINK;

use crate::core::{
    history::{Action, History, ModelAction},
    project::model::ModelId,
};

pub mod chart;
pub mod dragger;

pub fn labeled_separator(ui: &mut Ui, text: &str) {
    ui.horizontal(|ui| {
        let width = ui.fonts_mut(|f| {
            f.layout_no_wrap(text.into(), FontId::default(), Color32::default())
                .rect
                .width()
        });
        let spacing = ui.style().spacing.item_spacing.x;
        let bar = (ui.available_width() - width) / 2.0 - spacing;

        ui.add_sized([bar, 1.0], Separator::default().horizontal());
        ui.label(text);
        ui.add_sized([bar, 1.0], Separator::default().horizontal());
    });
}

pub fn grid(id_salt: &str) -> Grid {
    Grid::new(id_salt)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
}

/// Returns if the supplied widget response is being dragged or has focus.
pub fn being_edited(response: &Response) -> bool {
    response.dragged() || response.has_focus()
}

pub trait BeingEditedExt {
    fn being_edited(self, edited: &mut bool) -> Self;
}

impl BeingEditedExt for Response {
    fn being_edited(self, edited: &mut bool) -> Self {
        *edited |= self.dragged() || self.has_focus();
        self
    }
}

// todo: don't think the data stored through egui is ever being freed...
pub fn history_tracked_value(
    (edited, ui, history): (bool, &mut Ui, &mut History),
    (salt, value): (impl Hash, impl Fn() -> Action),
) {
    let id = ui.next_auto_id().with(salt);
    let old = ui.data_mut(|data| mem::replace(data.get_temp_mut_or(id, edited), edited));

    let data_id = id.with("data");
    (edited && !old).then(|| ui.data_mut(|data| data.insert_temp(data_id, value())));

    if (old && !edited)
        && let Some(old_value) = ui.data_mut(|data| data.get_temp::<Action>(data_id))
        && old_value != value()
    {
        ui.data_mut(|data| data.remove::<ModelAction>(data_id));
        history.track(old_value);
    }
}

pub fn history_tracked_model(
    (edited, ui, history): (bool, &mut Ui, &mut History),
    (model, value): (ModelId, impl Fn() -> ModelAction),
) {
    history_tracked_value(
        (edited, ui, history),
        (model, || Action::Model {
            id: model,
            action: value(),
        }),
    );
}

pub fn button_row<const N: usize>(ui: &mut Ui, mut buttons: [(&str, &mut dyn FnMut()); N]) {
    ui.horizontal(move |ui| {
        let spacing = ui.style().spacing.item_spacing.x;
        let count = buttons.len() as f32;
        let size = vec2(
            (ui.available_width() - spacing * (count - 1.0)) / count,
            0.0,
        );

        for (label, callback) in buttons.iter_mut() {
            if ui.add(Button::new(*label).min_size(size)).clicked() {
                callback();
            }
        }
    });
}

pub fn collapsing_toggle(
    title: &str,
    mut toggle: bool,
    content: impl FnOnce(&mut Ui),
    default: bool,
    ui: &mut Ui,
) -> bool {
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.checkbox(&mut toggle, "");
        CollapsingHeader::new(title)
            .default_open(default)
            .show(ui, |ui| content(ui));
    });
    toggle
}

pub fn link_button(ui: &mut Ui, text: &str, link: &str) {
    if ui.button((RichText::new(LINK).weak(), " ", text)).clicked() {
        ui.ctx().open_url(OpenUrl {
            url: link.to_owned(),
            new_tab: true, // doesn't matter
        });
    }
}
