use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use clone_macro::clone;
use common::{
    progress::Progress,
    serde::{ReaderDeserializer, WriterSerializer},
};
use tracing::info;

use crate::app::{
    project::Project,
    task::{
        BuildAccelerationStructures, MeshManifold, PollResult, Task, TaskApp, TaskStatus,
        thread::TaskThread,
    },
};

pub struct ProjectLoad {
    progress: Progress,
    path: PathBuf,
    handle: TaskThread<Project>,
}

pub struct ProjectSave {
    progress: Progress,
    path: PathBuf,
    handle: TaskThread<()>,
}

impl ProjectLoad {
    pub fn new(path: PathBuf) -> Self {
        let progress = Progress::new();

        info!("Loading project from `{}`", path.display());
        let handle = TaskThread::spawn(clone!([progress, path], move || {
            let file = File::open(&path).unwrap();
            let mut des = ReaderDeserializer::new(BufReader::new(file));
            Project::deserialize(&mut des, progress)
                .unwrap()
                .with_path(path)
        }));

        Self {
            progress,
            path,
            handle,
        }
    }
}

impl ProjectSave {
    pub fn new(project: Project, path: PathBuf) -> Self {
        let progress = Progress::new();

        info!("Saving project to `{}`", path.display());
        let handle = TaskThread::spawn(clone!([progress, path], move || {
            let file = File::create(path).unwrap();
            let mut ser = WriterSerializer::new(BufWriter::new(file));
            project.serialize(&mut ser, progress);
        }));

        Self {
            progress,
            path,
            handle,
        }
    }
}

impl Task for ProjectLoad {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        (self.handle.poll(app, "Failed to Load Project")).into_poll_result(|project| {
            app.config.add_recent_project(self.path.to_path_buf());
            *app.project = project;

            let mut result = PollResult::complete();
            let count = app.project.models.len();
            for (i, model) in app.project.models.iter_mut().enumerate() {
                model.update_oob(&app.project.slice_config.platform_size);
                result = result
                    .with_task(MeshManifold::new(model))
                    .with_task(BuildAccelerationStructures::new(model));

                info!(
                    " {} Loaded model `{}` with {} faces",
                    if i + 1 < count { "│" } else { "└" },
                    model.name,
                    model.mesh.face_count()
                );
            }

            result
        })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Loading Project".into(),
            details: Some(format!("Loading `{}`", file_name(&self.path))),
            progress: self.progress.progress(),
        })
    }
}

impl Task for ProjectSave {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        (self.handle.poll(app, "Failed to Save Project"))
            .into_poll_result(|_| PollResult::complete())
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Saving Project".into(),
            details: Some(format!("Saving `{}`", file_name(&self.path))),
            progress: self.progress.progress(),
        })
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}
