use std::{fs::File, io::BufReader, mem, thread::JoinHandle};

use anyhow::Result;
use common::{
    progress::Progress,
    serde::{ReaderDeserializer, SliceDeserializer},
};
use egui::{Context, Id, ProgressBar, Window, vec2};
use mesh_format::load_mesh;

use slicer::mesh::Mesh;
use tracing::info;

use crate::{
    app::{
        App,
        model::Model,
        task::{MeshManifold, Task},
    },
    ui::popup::{Popup, PopupIcon},
};

pub struct MeshLoad {
    progress: Progress,
    join: Option<JoinHandle<Result<mesh_format::Mesh>>>,
    name: String,
}

impl MeshLoad {
    pub fn file(file: File, name: String, format: &str) -> Self {
        let des = ReaderDeserializer::new(BufReader::new(file));
        let (progress, join) = load_mesh(des, format);
        Self {
            progress,
            join: Some(join),
            name,
        }
    }

    pub fn buffer(buffer: &'static [u8], name: String, format: &str) -> Self {
        let (progress, join) = load_mesh(SliceDeserializer::new(buffer), format);
        Self {
            progress,
            join: Some(join),
            name,
        }
    }
}

impl Task for MeshLoad {
    fn poll(&mut self, app: &mut App, ctx: &Context) -> bool {
        if self.progress.complete() {
            let handle = mem::take(&mut self.join).unwrap();
            let mesh = match handle.join().unwrap() {
                Ok(x) => x,
                Err(e) => {
                    app.popup.open(Popup::simple(
                        "Failed to Load Model",
                        PopupIcon::Error,
                        e.to_string(),
                    ));
                    return true;
                }
            };

            let mesh = Mesh::new(mesh.verts, mesh.faces);
            info!(
                "Loaded model `{}` with {} faces",
                self.name,
                mesh.face_count()
            );

            let mut rendered_mesh = Model::from_mesh(mesh)
                .with_name(mem::take(&mut self.name))
                .with_random_color();
            rendered_mesh.update_oob(&app.slice_config.platform_size);
            app.tasks.add(MeshManifold::new(&rendered_mesh));
            app.models.push(rendered_mesh);
            return true;
        }

        let size = vec2(400.0, 0.0);
        Window::new("")
            .id(Id::new(&self.name))
            .title_bar(false)
            .resizable(false)
            .default_size(size)
            .default_pos((ctx.content_rect().size() - size).to_pos2() / 2.0)
            .show(ctx, |ui| {
                ui.set_height(50.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Loading Model");
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Loading");
                    ui.monospace(&self.name);
                });
                ui.add(ProgressBar::new(self.progress.progress()).show_percentage())
            });

        false
    }
}
