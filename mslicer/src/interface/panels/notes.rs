use const_format::concatcp;
use egui::{Context, OutputCommand, ScrollArea, TextEdit, Ui, Widget};
use egui_phosphor::regular::{COPY, TRASH};

use crate::core::App;

const DESCRIPTION: &str =
    "Anything you write here will be saved into the project file for future reference.";

pub fn ui(app: &mut App, ui: &mut Ui, ctx: &Context) {
    ui.horizontal_wrapped(|ui| {
        (ui.button(concatcp!(COPY, " Copy")).clicked())
            .then(|| ctx.send_cmd(OutputCommand::CopyText(app.project.notes.clone())));

        ui.button(concatcp!(TRASH, " Clear"))
            .clicked()
            .then(|| app.project.notes.clear());
    });
    ui.add_space(8.0);

    ScrollArea::vertical().show(ui, |ui| {
        TextEdit::multiline(&mut app.project.notes)
            .hint_text(DESCRIPTION)
            .min_size(ui.available_size())
            .ui(ui);
    });
}
