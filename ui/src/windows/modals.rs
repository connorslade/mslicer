use eframe::Frame;
use egui::{Context, Grid, Window};

use crate::app::App;

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    Window::new("Modals")
        .open(&mut app.windows.show_modals)
        .show(ctx, |ui| {
            let mut meshes = app.meshes.write().unwrap();

            if meshes.is_empty() {
                ui.label("No modals loaded yet.");
                return;
            }

            Grid::new("modals")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    for (i, mesh) in meshes.iter_mut().enumerate() {
                        ui.label(&mesh.name);

                        ui.horizontal(|ui| {
                            mesh.hidden ^= ui.button(if mesh.hidden { "🗙" } else { "👁" }).clicked();

                            ui.collapsing("Details", |ui| {
                                Grid::new(format!("modal_{}", i))
                                    .num_columns(2)
                                    .spacing([40.0, 4.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label("Vertices");
                                        ui.monospace(mesh.mesh.vertices.len().to_string());
                                        ui.end_row();

                                        ui.label("Triangles");
                                        ui.monospace(mesh.mesh.faces.len().to_string());
                                        ui.end_row();
                                    });
                            });
                        });
                        ui.end_row()
                    }
                });
        });
}
