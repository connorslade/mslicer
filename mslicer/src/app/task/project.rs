use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    thread::{self, JoinHandle},
};

use clone_macro::clone;
use common::{
    progress::Progress,
    serde::{ReaderDeserializer, WriterSerializer},
};
use egui::Context;
use tracing::info;

use crate::app::{
    App,
    project::Project,
    task::{Task, TaskStatus},
};

pub struct ProjectLoad {
    progress: Progress,
    handle: Option<JoinHandle<Project>>,
}

pub struct ProjectSave {
    progress: Progress,
}

impl ProjectLoad {
    pub fn new(path: PathBuf) -> Self {
        let progress = Progress::new();

        info!("Loading project from `{}`", path.display());
        let handle = thread::spawn(clone!([progress], move || {
            let file = File::open(path).unwrap();
            let mut des = ReaderDeserializer::new(BufReader::new(file));
            Project::deserialize(&mut des, progress).unwrap()
        }));

        Self {
            progress,
            handle: Some(handle),
        }
    }
}

impl ProjectSave {
    pub fn new(project: Project, path: PathBuf) -> Self {
        let progress = Progress::new();

        info!("Saving project to `{}`", path.display());
        thread::spawn(clone!([progress], move || {
            let file = File::create(path).unwrap();
            let mut ser = WriterSerializer::new(BufWriter::new(file));
            project.serialize(&mut ser, progress);
        }));

        Self { progress }
    }
}

impl Task for ProjectLoad {
    fn poll(&mut self, app: &mut App, _ctx: &Context) -> bool {
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

        false
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Loading Project".into(),
            progress: self.progress.progress(),
        })
    }
}

impl Task for ProjectSave {
    fn poll(&mut self, _app: &mut App, _ctx: &Context) -> bool {
        self.progress.complete()
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Saving Project".into(),
            progress: self.progress.progress(),
        })
    }
}
