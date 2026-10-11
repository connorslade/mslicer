use egui::{TextEdit, Ui, Widget};

use crate::{
    core::App,
    interface::popup::{Popup, PopupApp},
};

const DESCRIPTION: &str =
    "Anything you write here will be saved into the project file for future reference.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new_seeded("Notes", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    TextEdit::multiline(&mut app.project.notes)
        .desired_width(ui.available_width())
        .ui(ui);
    false
}
