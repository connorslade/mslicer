use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread,
};

use eframe::Frame;
use egui::{Context, TopBottomPanel, Ui};
use rfd::FileDialog;
use slicer::slicer::slice_goo;

use crate::{app::{App, SliceProgress}, workspace::rendered_mesh::RenderedMesh};

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("mslicer");
            ui.separator();

            ui.menu_button("🖹 File", |ui| {
                if ui.button("Import Modal").clicked() {
                    // TODO: async
                    if let Some(path) = FileDialog::new().add_filter("STL", &["stl"]).pick_file() {
                        let name = path.file_name().unwrap().to_str().unwrap().to_string();

                        let mut file = std::fs::File::open(path).unwrap();
                        let modal = slicer::mesh::load_mesh(&mut file, "stl").unwrap();

                        app.meshes
                            .write()
                            .unwrap()
                            .push(RenderedMesh::from_mesh(modal).with_name(name));
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
                show_entry(ui, "Modals", &mut app.windows.show_modals);
                show_entry(ui, "Slice Config", &mut app.windows.show_slice_config);
                show_entry(ui, "Stats", &mut app.windows.show_stats);
                show_entry(ui, "Workspace", &mut app.windows.show_workspace);
            });

            ui.separator();

            if ui.button("Slice!").clicked() {
                let slice_config = app.slice_config.clone();
                let mesh = app.meshes.read().unwrap().first().unwrap().mesh.clone();

                let progress = Arc::new(SliceProgress {
                    current: AtomicU32::new(0),
                    total: AtomicU32::new(0),
                    result: Mutex::new(None),
                });
                app.slice_progress = Some(progress.clone());

                thread::spawn(move || {
                    let result = slice_goo(&slice_config, &mesh, |current, total| {
                        progress.current.store(current, Ordering::Relaxed);
                        progress.total.store(total, Ordering::Relaxed);
                    });
                    progress.result.lock().unwrap().replace(result);
                });
            }
        });
    });
}
