use eframe::Frame;
use egui::{CollapsingHeader, Context, Grid, Window};

use crate::{app::App, components::vec3_dragger};

enum Action {
    None,
    Remove(usize),
    Duplicate(usize),
}

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    Window::new("Models")
        .open(&mut app.windows.show_models)
        .show(ctx, |ui| {
            let mut meshes = app.meshes.write().unwrap();
            let mut action = Action::None;

            if meshes.is_empty() {
                ui.label("No models loaded yet.");
                return;
            }

            Grid::new("models")
                .num_columns(3)
                .striped(true)
                .show(ui, |ui| {
                    for (i, mesh) in meshes.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            mesh.hidden ^= ui
                                .button(if mesh.hidden { "🗙" } else { "👁" })
                                .on_hover_text(if mesh.hidden { "Show" } else { "Hide" })
                                .clicked();
                            ui.button("🗑")
                                .on_hover_text("Delete")
                                .clicked()
                                .then(|| action = Action::Remove(i));
                            ui.button("🗋")
                                .on_hover_text("Duplicate")
                                .clicked()
                                .then(|| action = Action::Duplicate(i));
                        });
                        ui.label(&mesh.name);

                        CollapsingHeader::new("Details")
                            .id_source(format!("model_details_{i}"))
                            .show(ui, |ui| {
                                Grid::new(format!("model_{}", i))
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label("Position");
                                        let mut position = mesh.mesh.position();
                                        vec3_dragger(ui, position.as_mut(), |x| x);
                                        (mesh.mesh.position() != position)
                                            .then(|| mesh.mesh.set_position(position));
                                        ui.end_row();

                                        ui.label("Scale");
                                        let mut scale = mesh.mesh.scale();
                                        vec3_dragger(ui, scale.as_mut(), |x| x.speed(0.01));
                                        (mesh.mesh.scale() != scale)
                                            .then(|| mesh.mesh.set_scale(scale));
                                        ui.end_row();

                                        ui.label("Rotation");
                                        let mut rotation = mesh.mesh.rotation();
                                        vec3_dragger(ui, rotation.as_mut(), |x| x.speed(0.01));
                                        (mesh.mesh.scale() != rotation)
                                            .then(|| mesh.mesh.set_rotation(rotation));
                                        ui.end_row();

                                        ui.label("Name");
                                        ui.text_edit_singleline(&mut mesh.name);
                                        ui.end_row();

                                        ui.label("Vertices");
                                        ui.monospace(mesh.mesh.vertices.len().to_string());
                                        ui.end_row();

                                        ui.label("Triangles");
                                        ui.monospace(mesh.mesh.faces.len().to_string());
                                        ui.end_row();
                                    });
                            });
                        ui.end_row()
                    }
                });

            match action {
                Action::Remove(i) => {
                    meshes.remove(i);
                }
                Action::Duplicate(i) => {
                    let mesh = meshes[i].clone();
                    meshes.push(mesh);
                }
                Action::None => {}
            }
        });
}
