use egui::{CollapsingHeader, Context, Grid, Ui};
use slicer::Pos;

use crate::{
    app::App,
    components::{vec3_dragger, vec3_dragger_proportional},
};

enum Action {
    None,
    Remove(usize),
    Duplicate(usize),
}

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let mut meshes = app.meshes.write();
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

                                ui.horizontal(|ui| {
                                    let mut scale = mesh.mesh.scale();
                                    if mesh.locked_scale {
                                        vec3_dragger_proportional(ui, scale.as_mut(), |x| {
                                            x.speed(0.01).clamp_range(0.001..=f32::MAX)
                                        });
                                    } else {
                                        vec3_dragger(ui, scale.as_mut(), |x| {
                                            x.speed(0.01).clamp_range(0.001..=f32::MAX)
                                        });
                                    }
                                    (mesh.mesh.scale() != scale)
                                        .then(|| mesh.mesh.set_scale(scale));

                                    mesh.locked_scale ^= ui
                                        .button(if mesh.locked_scale { "🔒" } else { "🔓" })
                                        .clicked();
                                });
                                ui.end_row();

                                ui.label("Rotation");
                                let mut rotation = rad_to_deg(mesh.mesh.rotation());
                                let original_rotation = rotation;
                                vec3_dragger(ui, rotation.as_mut(), |x| x);
                                (original_rotation != rotation)
                                    .then(|| mesh.mesh.set_rotation(deg_to_rad(rotation)));
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
}

fn rad_to_deg(pos: Pos) -> Pos {
    Pos::new(pos.x.to_degrees(), pos.y.to_degrees(), pos.z.to_degrees())
}

fn deg_to_rad(pos: Pos) -> Pos {
    Pos::new(pos.x.to_radians(), pos.y.to_radians(), pos.z.to_radians())
}
