use std::thread;

use clone_macro::clone;
use eframe::Frame;
use egui::{Context, TopBottomPanel, Ui};
use nalgebra::Vector2;
use rfd::FileDialog;
use slicer::{slicer::Slicer, Pos};

use crate::{
    app::{App, SliceResult},
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

                mesh.set_scale_unchecked(mesh.scale().component_mul(&Pos::new(
                    app.slice_config.platform_resolution.x as f32
                        / app.slice_config.platform_size.x,
                    app.slice_config.platform_resolution.y as f32
                        / app.slice_config.platform_size.y,
                    1.0,
                )));

                let (min, max) = mesh.minmax_point();
                let preview_scale = (app.slice_config.platform_size.x / (max.x - min.x))
                    .min(app.slice_config.platform_size.y / (max.y - min.y));

                let pos = mesh.position();
                mesh.set_position_unchecked(
                    pos + Pos::new(
                        app.slice_config.platform_resolution.x as f32 / 2.0,
                        app.slice_config.platform_resolution.y as f32 / 2.0,
                        pos.z - app.slice_config.slice_height,
                    ),
                );

                mesh.update_transformation_matrix();

                let slicer = Slicer::new(slice_config, mesh);
                app.slice_progress = Some(slicer.progress());

                thread::spawn(clone!([{ app.slice_result } as slice_result], move || {
                    let goo = slicer.slice();
                    slice_result.lock().unwrap().replace(SliceResult {
                        goo,
                        slice_preview_layer: 0,
                        last_preview_layer: 0,
                        preview_offset: Vector2::new(0.0, 0.0),
                        preview_scale,
                    });
                }));
            }
        });
    });
}
