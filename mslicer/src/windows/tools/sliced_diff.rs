use egui::{Button, DragValue, RichText, Ui, Widget};
use egui_phosphor::regular::{FOLDER_OPEN, STACK_SIMPLE};
use tools::sliced_diff::{Source, SourceId};

use crate::{
    app::App,
    generator_tool,
    task::FileDialog,
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "todo.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Sliced Diff", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();

    grid("").show(ui, |ui| {
        ui.label("Old");
        source_ui(app, ui, SourceId::Old);
        ui.end_row();

        ui.label("New");
        source_ui(app, ui, SourceId::New);
        ui.end_row();

        let tool = &mut app.state.tools.sliced_diff;
        ui.label("Threshold");
        ui.horizontal(|ui| {
            DragValue::new(&mut tool.threshold).ui(ui);
            ui.take_available_width();
        });
        ui.end_row();
    });

    ui.add_space(8.0);
    ui.centered_and_justified(|ui| {
        let tool = &mut app.state.tools.sliced_diff;
        if ui
            .add_enabled(
                !slicing && tool.sources_specified(),
                Button::new("Generate"),
            )
            .clicked()
        {
            generator_tool!(app, tool);
        }
    });

    false
}

fn source_ui(app: &mut PopupApp, ui: &mut Ui, id: SourceId) {
    ui.horizontal(|ui| {
        let source = app.state.tools.sliced_diff.source_mut(id);
        if ui.button(STACK_SIMPLE).clicked() {
            *source = Source::Loaded;
        }

        if ui.button(FOLDER_OPEN).clicked() {
            app.tasks.add(FileDialog::pick_file(
                ("Sliced", &["goo", "ctb", "nanodlp"]),
                move |app, path, _tasks| {
                    *app.state.tools.sliced_diff.source_mut(id) = Source::File(path.to_path_buf());
                },
            ));
        }

        match source {
            Source::Empty => {
                ui.label("Unspecified");
            }
            Source::File(path) => {
                let name = path.file_name().unwrap().to_string_lossy();
                ui.label(RichText::new(name).underline())
                    .on_hover_text(path.to_string_lossy());
            }
            Source::Loaded => {
                ui.label("Loaded");
            }
        };
    });
}
