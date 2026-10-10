use std::{fs::File, io::BufReader, sync::Arc};

use egui::{Button, DragValue, RichText, Ui, Widget, vec2};
use encase::vector::AsMutVectorParts;
use image::ImageFormat;

use crate::{
    core::App,
    interface::{
        components::{dragger, grid},
        popup::{Popup, PopupApp},
    },
    mesh_generator_tool,
    task::FileDialog,
};

const DESCRIPTION: &str = "Generate a mesh from a grayscale height map image.";
const SUPPORTED_FORMATS: &[&str] = &[
    "avif", "jpg", "jpeg", "jfif", "png", "apng", "gif", "webp", "tif", "tiff", "tga", "dds",
    "bmp", "ico", "hdr", "exr", "pbm", "pam", "ppm", "pgm", "ff", "qoi", "pcx",
];

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new_seeded("Height Map", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.height_map;

    ui.horizontal(|ui| {
        if ui.button("Load Image").clicked() {
            app.tasks.add(FileDialog::pick_file(
                ("Image", SUPPORTED_FORMATS),
                |app, path, _tasks| {
                    let format = ImageFormat::from_path(path).unwrap();
                    let file = BufReader::new(File::open(path).unwrap());
                    let image = image::load(file, format).unwrap().to_luma32f();

                    app.state.tools.height_map.data = Some(Arc::new((image, path.to_path_buf())));
                },
            ));
        }

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
        ui.label("Size (mm)");
        ui.horizontal(|ui| {
            dragger::vec2(ui, tool.size.as_mut_parts(), |x| x.speed(0.1));
            ui.take_available_width();
        });
        ui.end_row();

        ui.label("Max Height");
        DragValue::new(&mut tool.max_height)
            .speed(0.1)
            .suffix(" mm")
            .ui(ui);
        ui.end_row();
    });
    ui.add_space(8.0);

    ui.vertical_centered(|ui| {
        let button = Button::new("Generate").min_size(vec2(ui.available_width(), 0.0));
        if ui
            .add_enabled(!slicing && tool.data.is_some(), button)
            .clicked()
        {
            mesh_generator_tool!(app, tool, "Height Map");
        }
    });

    false
}
