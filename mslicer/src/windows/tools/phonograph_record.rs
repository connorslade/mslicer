use common::units::Mircometer;
use egui::{Button, CollapsingHeader, DragValue, Ui, Widget, vec2};

use crate::{
    app::App,
    task::{FileDialog, MeshLoad},
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "Generates a phonographic record mesh from an audio file.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Phonograph Record", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let slicing = app.is_slicing();
    let tool = &mut app.state.tools.phonograph_record;

    let audio_loaded = !tool.audio.as_os_str().is_empty();
    ui.horizontal(|ui| {
        if ui.button("Load Audio").clicked() {
            app.tasks.add(FileDialog::pick_file(
                ("Waveform Audio", &["wav"]),
                |app, path, _tasks| {
                    app.state.tools.phonograph_record.audio = path.to_path_buf();
                },
            ));
        }

        if audio_loaded {
            let name = tool.audio.file_name().unwrap().to_string_lossy();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Loaded ");
                ui.weak(name).on_hover_text(tool.audio.to_string_lossy());
                ui.label(".");
            });
        }
    });

    ui.add_space(8.0);
    CollapsingHeader::new("Disk Size")
        .default_open(true)
        .show(ui, |ui| {
            grid("disk").show(ui, |ui| {
                ui.label("Outer Radius");
                DragValue::new(tool.outer_radius.raw_mut())
                    .suffix(" mm")
                    .ui(ui);
                ui.end_row();

                ui.label("Inner Radius");
                DragValue::new(tool.inner_radius.raw_mut())
                    .suffix(" mm")
                    .ui(ui);
                ui.end_row();

                ui.label("Thickness");
                ui.horizontal(|ui| {
                    DragValue::new(tool.thickness.raw_mut())
                        .suffix(" mm")
                        .ui(ui);
                    ui.take_available_width();
                });
                ui.end_row();
            });
        });

    CollapsingHeader::new("Groove")
        .default_open(true)
        .show(ui, |ui| {
            grid("groove").show(ui, |ui| {
                ui.label("Pitch");
                tool.pitch.with::<Mircometer>(|x| {
                    DragValue::new(x).suffix(" μm").ui(ui);
                });
                ui.end_row();

                ui.label("Width");
                tool.width.with::<Mircometer>(|x| {
                    DragValue::new(x).suffix(" μm").ui(ui);
                });
                ui.end_row();

                ui.label("Resolution");
                DragValue::new(&mut tool.groove_resolution)
                    .suffix(" Hz")
                    .ui(ui);
                ui.end_row();

                ui.label("Playback Speed");
                DragValue::new(&mut tool.rpm).suffix(" RPM").ui(ui);
                ui.end_row();

                ui.label("Modulation");
                ui.horizontal(|ui| {
                    // todo: store as percent
                    DragValue::new(&mut tool.modulation)
                        .custom_formatter(|n, _| format!("{:.2}", n * 100.0))
                        .custom_parser(|s| s.parse::<f64>().map(|x| x / 100.0).ok())
                        .suffix("%")
                        .ui(ui);
                    ui.take_available_width();
                });
                ui.end_row();
            });
        });

    ui.add_space(8.0);

    // todo: make this a component
    ui.vertical_centered(|ui| {
        let button = Button::new("Generate").min_size(vec2(ui.available_width(), 0.0));
        if ui.add_enabled(!slicing && audio_loaded, button).clicked() {
            // todo: generate async
            let mesh = tool.generate().unwrap();
            app.tasks.add(MeshLoad::complete("Record".to_owned(), mesh));
        }
    });

    false
}
