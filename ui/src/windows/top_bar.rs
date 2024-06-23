use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread,
};

use eframe::Frame;
use egui::{Context, TopBottomPanel, Ui};
use nalgebra::Vector2;
use rfd::FileDialog;
use slicer::{slicer::slice_goo, Pos};

use crate::{
    app::{App, SliceProgress, SliceResult},
    render::rendered_mesh::RenderedMesh,
};

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("mslicer");
            ui.separator();

            ui.menu_button("🖹 File", |ui| {
                if ui.button("Import Model").clicked() {
                    // TODO: async
                    if let Some(path) = FileDialog::new().add_filter("STL", &["stl"]).pick_file() {
                        let name = path.file_name().unwrap().to_str().unwrap().to_string();

                        let mut file = std::fs::File::open(path).unwrap();
                        let model = slicer::mesh::load_mesh(&mut file, "stl").unwrap();

                        app.meshes
                            .write()
                            .unwrap()
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

            if ui.button("Slice!").clicked() {
                let slice_config = app.slice_config.clone();
                let mut mesh = app.meshes.read().unwrap().first().unwrap().mesh.clone();
                mesh.scale = mesh.scale.component_mul(&Pos::new(
                    app.slice_config.platform_resolution.x as f32
                        / app.slice_config.platform_size.x,
                    app.slice_config.platform_resolution.y as f32
                        / app.slice_config.platform_size.y,
                    1.0,
                ));

                let (min, max) = mesh.minmax_point();
                let preview_scale = (app.slice_config.platform_size.x / (max.x - min.x))
                    .min(app.slice_config.platform_size.y / (max.y - min.y));

                mesh.position += Pos::new(
                    app.slice_config.platform_resolution.x as f32 / 2.0,
                    app.slice_config.platform_resolution.y as f32 / 2.0,
                    mesh.position.z - app.slice_config.slice_height,
                );

                let progress = Arc::new(SliceProgress {
                    current: AtomicU32::new(0),
                    total: AtomicU32::new(0),
                    result: Mutex::new(None),
                });
                app.slice_progress = Some(progress.clone());

                thread::spawn(move || {
                    let goo = slice_goo(&slice_config, &mesh, |current, total| {
                        progress.current.store(current, Ordering::Relaxed);
                        progress.total.store(total, Ordering::Relaxed);
                    });
                    progress.result.lock().unwrap().replace(SliceResult {
                        goo,
                        slice_preview_layer: 0,
                        last_preview_layer: 0,
                        preview_offset: Vector2::new(0.0, 0.0),
                        preview_scale,
                    });
                });
            }
        });
    });
}
