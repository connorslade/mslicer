use std::{fs::File, io::BufReader};

use egui::{Align, Context, Layout, TopBottomPanel};
use rfd::FileDialog;
use tracing::info;

use crate::{app::App, render::rendered_mesh::RenderedMesh, windows::Tab};

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
                        let format = ext
                            .expect("Selected file has no extension")
                            .to_string_lossy();

                        let file = File::open(&path).unwrap();
                        let mut buf = BufReader::new(file);
                        let model = slicer::mesh::load_mesh(&mut buf, &format).unwrap();
                        info!("Loaded model `{name}` with {} faces", model.faces.len());

                        app.meshes
                            .write()
                            .push(RenderedMesh::from_mesh(model).with_name(name));
                    }
                }

                let _ = ui.button("Save Project");
                let _ = ui.button("Load Project");
            });

            ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                let slicing = match &app.slice_operation {
                    Some(operation) => operation.progress.completed() < operation.progress.total(),
                    None => false,
                };
                ui.add_enabled_ui(!slicing, |ui| {
                    if ui.button("Slice!").clicked() {
                        app.slice();
                        app.dock_state.add_window(vec![Tab::SliceOperation]);
                    }
                });
            });
        });
    });
}
