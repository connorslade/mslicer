use std::{fs::File, io::BufReader};

use eframe::Frame;
use egui::{Context, TopBottomPanel, Ui};
use rfd::FileDialog;
use tracing::info;

use crate::{app::App, render::rendered_mesh::RenderedMesh};

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("mslicer");
            ui.separator();

            ui.menu_button("🖹 File", |ui| {
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
            });

            ui.menu_button("🖹 View", |ui| {
                fn show_entry(ui: &mut Ui, name: &str, show: &mut bool) {
                    *show ^= ui
                        .button(format!("{} {name}", if *show { "👁" } else { "🗙" }))
                        .clicked();
                }

                if ui.button("Organize windows").clicked() {
                    ui.ctx().memory_mut(|mem| mem.reset_areas());
                }

                ui.separator();

                show_entry(ui, "About", &mut app.windows.show_about);
                show_entry(ui, "Model", &mut app.windows.show_models);
                show_entry(ui, "Slice Config", &mut app.windows.show_slice_config);
                show_entry(ui, "Stats", &mut app.windows.show_stats);
                show_entry(ui, "Workspace", &mut app.windows.show_workspace);
            });

            ui.separator();

            let slicing = match &app.slice_operation {
                Some(operation) => operation.progress.completed() < operation.progress.total(),
                None => false,
            };
            ui.add_enabled_ui(!slicing, |ui| {
                ui.button("Slice!").clicked().then(|| app.slice());
            });
        });
    });
}
