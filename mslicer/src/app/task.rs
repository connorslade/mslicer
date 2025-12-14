use std::{fs::File, io::BufReader, mem, thread::JoinHandle};

use common::serde::{ReaderDeserializer, SliceDeserializer};
use egui::{vec2, Context, Id, ProgressBar, Window};
use mesh_format::load_mesh;
use slicer::mesh::Mesh;
use tracing::info;

use crate::{app::App, render::rendered_mesh::RenderedMesh};

// Async operation that can be polled every frame.
pub trait Task {
    /// Returns true if the task has completed.
    fn poll(&mut self, app: &mut App, ctx: &Context) -> bool;
}

#[derive(Default)]
pub struct TaskManager {
    tasks: Vec<Box<dyn Task>>,
}

impl TaskManager {
    pub fn add(&mut self, task: impl Task + 'static) {
        self.tasks.push(Box::new(task));
    }

    pub(super) fn render(&mut self, app: &mut App, ctx: &Context) {
        let mut i = 0;
        while i < self.tasks.len() {
            let task = &mut self.tasks[i];
            if task.poll(app, ctx) {
                self.tasks.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

pub struct MeshLoad {
    progress: mesh_format::Progress,
    join: Option<JoinHandle<mesh_format::Mesh>>,
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
            let mesh = handle.join().unwrap();
            let mut mesh = Mesh::new(mesh.verts, mesh.faces, Vec::new());
            info!(
                "Loaded model `{}` with {} faces",
                self.name,
                mesh.face_count()
            );

            mesh.recompute_normals();
            let mut rendered_mesh = RenderedMesh::from_mesh(mesh)
                .with_name(mem::take(&mut self.name))
                .with_random_color();
            rendered_mesh.update_oob(&app.slice_config);
            app.meshes.write().push(rendered_mesh);
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
