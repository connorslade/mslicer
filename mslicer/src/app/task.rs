use std::{
    fs::File,
    io::BufReader,
    mem,
    path::Path,
    thread::{self, JoinHandle},
};

use anyhow::Result;
use clone_macro::clone;
use common::{
    progress::Progress,
    serde::{ReaderDeserializer, SliceDeserializer},
};
use egui::{Context, Id, ProgressBar, Window, vec2};
use mesh_format::load_mesh;
use poll_promise::Promise;
use rfd::{AsyncFileDialog, FileHandle};
use slicer::mesh::Mesh;
use tracing::info;

use crate::{
    app::App,
    render::model::{MeshWarnings, Model},
    ui::popup::{Popup, PopupIcon},
};

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
            let mut task = self.tasks.remove(i);
            if !task.poll(app, ctx) {
                self.tasks.push(task);
                i += 1;
            }
        }
    }
}

pub struct FileDialog {
    func: Option<Box<dyn FnOnce(&mut App, &Path)>>,
    promise: Promise<Option<FileHandle>>,
}

impl FileDialog {
    fn new(
        file: impl Future<Output = Option<FileHandle>> + Send + 'static,
        callback: impl FnMut(&mut App, &Path) + 'static,
    ) -> Self {
        let promise = Promise::spawn_async(file);
        FileDialog {
            func: Some(Box::new(callback)),
            promise,
        }
    }

    pub fn pick_file(
        (name, extensions): (impl Into<String>, &[impl ToString]),
        callback: impl FnMut(&mut App, &Path) + 'static,
    ) -> Self {
        let file = AsyncFileDialog::new()
            .add_filter(name, extensions)
            .pick_file();
        Self::new(file, callback)
    }

    pub fn save_file(
        (name, extensions): (impl Into<String>, &[impl ToString]),
        callback: impl FnMut(&mut App, &Path) + 'static,
    ) -> Self {
        let file = AsyncFileDialog::new()
            .add_filter(name, extensions)
            .save_file();
        Self::new(file, callback)
    }
}

impl Task for FileDialog {
    fn poll(&mut self, app: &mut App, _ctx: &Context) -> bool {
        let result = self.promise.ready();
        if let Some(data) = result
            && let Some(handle) = data
        {
            let path = handle.path();
            self.func.take().unwrap()(app, path);
        }

        result.is_some()
    }
}

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
            rendered_mesh.update_oob(&app.slice_config);
            app.tasks.add(MeshManifold::new(&rendered_mesh));
            app.models.write().push(rendered_mesh);
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

pub struct MeshManifold {
    mesh: u32,
    join: Option<JoinHandle<bool>>,
}

impl MeshManifold {
    pub fn new(mesh: &Model) -> Self {
        Self {
            mesh: mesh.id,
            join: Some(thread::spawn(clone!([{ mesh.mesh } as model], move || {
                model.is_manifold()
            }))),
        }
    }
}

impl Task for MeshManifold {
    fn poll(&mut self, app: &mut App, _ctx: &Context) -> bool {
        if self.join.as_ref().unwrap().is_finished() {
            let result = mem::take(&mut self.join).unwrap().join().unwrap();
            if let Some(model) = app.models.write().iter_mut().find(|x| x.id == self.mesh) {
                model.warnings.set(MeshWarnings::NonManifold, !result);
            }

            true
        } else {
            false
        }
    }
}
