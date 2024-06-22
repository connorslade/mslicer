use eframe::Frame;
use egui::{Context, Grid, Window};

use crate::{app::App, components::vec3_dragger};

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    Window::new("Models")
        .open(&mut app.windows.show_models)
        .show(ctx, |ui| {
            let mut meshes = app.meshes.write().unwrap();

            if meshes.is_empty() {
                ui.label("No models loaded yet.");
                return;
            }

            Grid::new("models")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for (i, mesh) in meshes.iter_mut().enumerate() {
                        ui.label(&mesh.name);

                        ui.horizontal(|ui| {
                            mesh.hidden ^= ui.button(if mesh.hidden { "🗙" } else { "👁" }).clicked();

                            ui.collapsing("Details", |ui| {
                                Grid::new(format!("model_{}", i))
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label("Position");
                                        vec3_dragger(ui, mesh.mesh.position.as_mut(), |x| x);
                                        ui.end_row();

                                        ui.label("Scale");
                                        vec3_dragger(ui, mesh.mesh.scale.as_mut(), |x| {
                                            x.speed(0.01)
                                        });
                                        ui.end_row();

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
