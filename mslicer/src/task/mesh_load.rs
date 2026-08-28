use std::{fs::File, io::BufReader, mem, path::PathBuf};

use anyhow::Result;
use clone_macro::clone;
use common::{
    progress::Progress,
    serde::{ReaderDeserializer, SliceDeserializer},
};
use mesh_format::load_mesh;

use slicer::mesh::Mesh;
use tracing::info;

use crate::{
    app::history::Action,
    project::model::Model,
    task::{
        MeshManifold, PollResult, Task, TaskApp, TaskStatus,
        acceleration_structures::BuildAccelerationStructures, thread::TaskThread,
    },
};

pub struct MeshLoad {
    progress: Progress,
    join: TaskThread<Mesh>,

    name: String,
    file: Option<PathBuf>,
}

impl MeshLoad {
    pub fn file(path: PathBuf, name: String, format: String) -> Result<Self> {
        let file = File::open(&path)?;
        let des = ReaderDeserializer::new(BufReader::new(file));

        let progress = Progress::new();
        Ok(Self {
            join: TaskThread::spawn(clone!([progress], move || {
                let mesh = load_mesh(des, &format, progress).unwrap();
                Mesh::new(mesh.verts, mesh.faces)
            })),
            progress,

            name,
            file: Some(path),
        })
    }

    pub fn buffer(buffer: &'static [u8], name: String, format: String) -> Self {
        let des = SliceDeserializer::new(buffer);
        let progress = Progress::new();
        Self {
            join: TaskThread::spawn(clone!([progress], move || {
                let mesh = load_mesh(des, &format, progress).unwrap();
                Mesh::new(mesh.verts, mesh.faces)
            })),
            progress,

            name,
            file: None,
        }
    }

    pub fn complete(name: String, mesh: Mesh) -> Self {
        Self {
            progress: Progress::already_complete(),
            join: TaskThread::spawn(|| mesh),

            name,
            file: None,
        }
    }
}

impl Task for MeshLoad {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        (self.join.poll(app, "Failed to Load Model")).into_poll_result(|mesh| {
            info!(
                "Loaded model `{}` with {} faces",
                self.name,
                mesh.face_count()
            );

            let mut model = Model::from_mesh(mesh)
                .with_name(mem::take(&mut self.name))
                .width_file(self.file.take())
                .with_random_color();
            model.update_oob(&app.project.slice_config.platform_size);
            let result = PollResult::complete()
                .with_task(MeshManifold::new(&model))
                .with_task(BuildAccelerationStructures::new(&model));
            app.history.track(Action::ModelAdded { id: model.id });
            app.project.models.push(model);
            result
        })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Loading Model".into(),
            details: Some(format!("Loading `{}`", self.name)),
            progress: self.progress.progress(),
        })
    }
}
