use std::{fs::File, io::BufReader, sync::Arc};

use common::units::Milimeters;
use egui::{Button, ComboBox, DragValue, RichText, Ui, Widget, vec2};
use image::ImageFormat;
use tools::{
    image::{Flip, Rotation, Sizing},
    misc::alignment::Alignment,
};

use crate::{
    generator_tool,
    interface::{components::grid, popup::PopupApp},
    task::FileDialog,
    util::IMAGE_FORMATS,
};

pub const NAME: &str = "Image Layer";
pub const DESCRIPTION: &str = "Generate a single layer exposure from an image file. Useful for exposing things other than resin.";

pub fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
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
        ui.label("Size");
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("sizing")
                .selected_text(tool.size.name())
                .show_ui(ui, |ui| {
                    let width = Milimeters::new(10.0);
                    let height = Milimeters::new(10.0);

                    for size in [
                        Sizing::Printer,
                        Sizing::Width(width),
                        Sizing::Height(height),
                        Sizing::Both { width, height },
                    ] {
                        ui.selectable_value(&mut tool.size, size.clone(), size.name());
                    }
                });

            match &mut tool.size {
                Sizing::Printer => {}
                Sizing::Width(length) | Sizing::Height(length) => {
                    DragValue::new(length.raw_mut()).suffix(" mm").ui(ui);
                }
                Sizing::Both { width, height } => {
                    DragValue::new(width.raw_mut()).ui(ui);
                    ui.label("×");
                    DragValue::new(height.raw_mut()).ui(ui);
                    ui.label("(mm)");
                }
            }
            ui.take_available_width();
        });
        ui.end_row();

        ui.label("Rotation");
        ComboBox::from_id_salt("rotation")
            .selected_text(tool.rotation.name())
            .show_ui(ui, |ui| {
                for rotation in Rotation::ALL {
                    ui.selectable_value(&mut tool.rotation, rotation, rotation.name());
                }
            });
        ui.end_row();

        ui.label("Flip");
        ComboBox::from_id_salt("flip")
            .selected_text(tool.flip.name())
            .show_ui(ui, |ui| {
                for flip in Flip::ALL {
                    ui.selectable_value(&mut tool.flip, flip, flip.name());
                }
            });
        ui.end_row();

        ui.label("Alignment");
        ComboBox::from_id_salt("alignment")
            .selected_text(tool.alignment.name())
            .show_ui(ui, |ui| {
                for alignment in Alignment::ALL {
                    ui.selectable_value(&mut tool.alignment, alignment, alignment.name());
                }
            });
        ui.end_row();

        ui.label("Background");
        DragValue::new(&mut tool.background).ui(ui);
        ui.end_row();
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
