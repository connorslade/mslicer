use std::{fs::File, io::BufReader, sync::Arc};

use egui::{Button, RichText, Ui, vec2};
use image::ImageFormat;

use crate::{
    core::App,
    generator_tool,
    interface::{
        components::grid,
        popup::{Popup, PopupApp},
    },
    task::FileDialog,
    util::IMAGE_FORMATS,
};

const DESCRIPTION: &str = "todo";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new_seeded("Image", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.image;

    ui.horizontal(|ui| {
        if ui.button("Load Image").clicked() {
            app.tasks.add(FileDialog::pick_file(
                ("Image", IMAGE_FORMATS),
                |app, path, _tasks| {
                    let format = ImageFormat::from_path(path).unwrap();
                    let file = BufReader::new(File::open(path).unwrap());
                    let image = image::load(file, format).unwrap().to_luma8();

                    app.state.tools.image.data = Some(Arc::new((image, path.to_path_buf())));
                },
            ));
        }

        // todo: make this a component, i think its also used in height map and phonograph record
        if let Some(path) = tool.data.as_ref().map(|x| &x.1) {
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
    grid("").show(ui, |ui| {
        // todo
    });
    ui.add_space(8.0);

    ui.vertical_centered(|ui| {
        let button = Button::new("Generate").min_size(vec2(ui.available_width(), 0.0));
        if ui
            .add_enabled(!slicing && tool.data.is_some(), button)
            .clicked()
        {
            generator_tool!(app, tool);
        }
    });

    false
}
