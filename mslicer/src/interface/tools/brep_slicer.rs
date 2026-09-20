use egui::{Button, RichText, Ui, vec2};
use egui_phosphor::regular::FILE;

use crate::{
    app::App,
    generator_tool,
    task::FileDialog,
    ui::popup::{Popup, PopupApp},
};

const DESCRIPTION: &str =
    "An experiment in slicing models directly from a boundary representation (BREP) like .step.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new_seeded("Brep Slicer", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.brep_slicer;

    ui.horizontal(|ui| {
        if ui.button(FILE).clicked() {
            app.tasks.add(FileDialog::pick_file(
                ("Step", &["step"]),
                |app, path, _tasks| {
                    app.state.tools.brep_slicer.step = Some(path.to_path_buf());
                },
            ));
        }

        if let Some(path) = &tool.step {
            let name = path.file_name().unwrap().to_string_lossy();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Loaded ");
                ui.label(RichText::new(name).underline())
                    .on_hover_text(path.to_string_lossy());
                ui.label(".");
            });
        }
    });

    ui.add_space(8.0);
    ui.vertical_centered(|ui| {
        let button = Button::new("Generate").min_size(vec2(ui.available_width(), 0.0));
        if ui.add_enabled(!slicing, button).clicked() {
            generator_tool!(app, tool);
        }
    });

    false
}
