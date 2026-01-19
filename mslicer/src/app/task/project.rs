use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

use clone_macro::clone;
use common::{
    progress::Progress,
    serde::{ReaderDeserializer, WriterSerializer},
};
use egui::{Context, Id};
use tracing::info;

use crate::app::{
    App,
    project::Project,
    task::{Task, progress_window},
};

pub struct ProjectLoad {
    progress: Progress,
    handle: Option<JoinHandle<Project>>,
    name: String,
}

pub struct ProjectSave {
    progress: Progress,
    name: String,
}

impl ProjectLoad {
    pub fn new(path: PathBuf) -> Self {
        let progress = Progress::new();
        let name = file_name(&path);

        info!("Loading project from `{}`", path.display());
        let handle = thread::spawn(clone!([progress], move || {
            let file = File::open(path).unwrap();
            let mut des = ReaderDeserializer::new(BufReader::new(file));
            Project::deserialize(&mut des, progress).unwrap()
        }));

        Self {
            progress,
            handle: Some(handle),
            name,
        }
    }
}

impl ProjectSave {
    pub fn new(project: Project, path: PathBuf) -> Self {
        let progress = Progress::new();
        let name = file_name(&path);

        info!("Saving project to `{}`", path.display());
        thread::spawn(clone!([progress], move || {
            let file = File::create(path).unwrap();
            let mut ser = WriterSerializer::new(BufWriter::new(file));
            project.serialize(&mut ser, progress);
        }));

        Self { progress, name }
    }
}

impl Task for ProjectLoad {
    fn poll(&mut self, app: &mut App, ctx: &Context) -> bool {
        if self.progress.complete() {
            app.project = self.handle.take().unwrap().join().unwrap();

            let count = app.project.models.len();
            for (i, mesh) in app.project.models.iter().enumerate() {
                info!(
                    " {} Loaded model `{}` with {} faces",
                    if i + 1 < count { "│" } else { "└" },
                    mesh.name,
                    mesh.mesh.face_count()
                );
            }

            return true;
        }

        progress_window(
            ctx,
            Id::new(&self.name),
            &self.progress,
            "Loading Project",
            |ui| {
                ui.horizontal(|ui| {
                    ui.label("Saving");
                    ui.monospace(&self.name);
                });
            },
        );

        false
    }
}

impl Task for ProjectSave {
    fn poll(&mut self, _app: &mut App, ctx: &Context) -> bool {
        progress_window(
            ctx,
            Id::new(&self.name),
            &self.progress,
            "Saving Project",
            |ui| {
                ui.horizontal(|ui| {
                    ui.label("Saving");
                    ui.monospace(&self.name);
                });
            },
        );

        self.progress.complete()
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}
