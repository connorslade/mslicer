use std::{
    fs::File,
    io::{BufReader, Cursor},
};

use egui::{Align, Context, Layout, TopBottomPanel};
use rfd::FileDialog;

use crate::app::App;

pub fn ui(app: &mut App, ctx: &Context) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("mslicer");
            ui.separator();

            ui.menu_button("🖹 File", |ui| {
                ui.style_mut().visuals.button_frame = false;
                if ui.button("Import Model").clicked() {
                    // TODO: async
                    if let Some(path) = FileDialog::new()
                        .add_filter("Mesh", &["stl", "obj"])
                        .pick_file()
                    {
                        let name = path.file_name().unwrap().to_str().unwrap().to_string();
                        let ext = path.extension();
                        let format = ext.unwrap_or_default().to_string_lossy();

                        let file = File::open(&path).unwrap();
                        let mut buf = BufReader::new(file);
                        app.load_mesh(&mut buf, &format, name);
                    }
                }

                if ui.button("Load Utah Teapot").clicked() {
                    let mut buf = Cursor::new(include_bytes!("../assets/teapot.stl"));
                    app.load_mesh(&mut buf, "stl", "Utah Teapot".into());
                }

                ui.separator();

                let _ = ui.button("Save Project");
                let _ = ui.button("Load Project");
            });

            ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                let slicing = match &app.slice_operation {
                    Some(operation) => operation.progress.completed() < operation.progress.total(),
                    None => false,
                };
                ui.add_enabled_ui(!slicing, |ui| {
                    ui.button("Slice!").clicked().then(|| app.slice());
                });
            });
        });
    });
}
