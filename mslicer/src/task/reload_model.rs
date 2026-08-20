use std::{fs::File, io::BufReader, mem, path::PathBuf};

use anyhow::Context;
use clone_macro::clone;
use common::{progress::Progress, serde::ReaderDeserializer};
use mesh_format::load_mesh;

use slicer::mesh::Mesh;
use tracing::info;

use crate::{
    project::model::ModelId,
    task::{
        MeshManifold, PollResult, Task, TaskApp, TaskStatus,
        acceleration_structures::BuildAccelerationStructures, thread::TaskThread,
    },
};

pub struct ReloadModel {
    progress: Progress,
    join: TaskThread<Mesh>,

    model: ModelId,
    path: PathBuf,
    name: String,
}

impl ReloadModel {
    pub fn new(model: ModelId, name: String, path: PathBuf) -> Self {
        let ext = (path.extension().context("Unspecified file type").unwrap())
            .to_string_lossy()
            .into_owned();

        let file = File::open(&path).unwrap();
        let des = ReaderDeserializer::new(BufReader::new(file));

        let progress = Progress::new();
        Self {
            join: TaskThread::spawn(clone!([progress], move || {
                let mesh = load_mesh(des, &ext, progress).unwrap();
                Mesh::new(mesh.verts, mesh.faces)
            })),
            progress,

            model,
            path,
            name,
        }
    }
}

impl Task for ReloadModel {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        (self.join.poll(app, "Failed to Load Model")).into_poll_result(|mesh| {
            info!(
                "Reloaded model `{}` with {} faces",
                self.name,
                mesh.face_count()
            );

            let platform_size = app.project.slice_config.platform_size;
            if let Some(model) = app.project.model(self.model) {
                model.replace_mesh(mesh, mem::take(&mut self.path), &platform_size);
                PollResult::complete()
                    .with_task(MeshManifold::new(model))
                    .with_task(BuildAccelerationStructures::new(model))
            } else {
                PollResult::complete()
            }
        })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Reloading Model".into(),
            details: Some(format!("Loading `{}`", self.name)),
            progress: self.progress.progress(),
        })
    }
}
