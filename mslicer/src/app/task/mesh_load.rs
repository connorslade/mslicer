use std::{fs::File, io::BufReader, mem};

use clone_macro::clone;
use common::{
    progress::Progress,
    serde::{ReaderDeserializer, SliceDeserializer},
};
use mesh_format::load_mesh;

use slicer::mesh::Mesh;
use tracing::info;

use crate::app::{
    project::model::Model,
    task::{
        MeshManifold, PollResult, Task, TaskApp, TaskStatus,
        acceleration_structures::BuildAccelerationStructures, thread::TaskThread,
    },
};

pub struct MeshLoad {
    progress: Progress,
    join: TaskThread<mesh_format::Mesh>,
    name: String,
}

impl MeshLoad {
    pub fn file(file: File, name: String, format: String) -> Self {
        let des = ReaderDeserializer::new(BufReader::new(file));
        let progress = Progress::new();
        Self {
            join: TaskThread::spawn(clone!([progress], move || {
                load_mesh(des, &format, progress).unwrap()
            })),
            progress,
            name,
        }
    }

    pub fn buffer(buffer: &'static [u8], name: String, format: String) -> Self {
        let des = SliceDeserializer::new(buffer);
        let progress = Progress::new();
        Self {
            join: TaskThread::spawn(clone!([progress], move || {
                load_mesh(des, &format, progress).unwrap()
            })),
            progress,
            name,
        }
    }
}

impl Task for MeshLoad {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        (self.join.poll(app, "Failed to Load Model")).into_poll_result(|mesh| {
            let mesh = Mesh::new(mesh.verts, mesh.faces);
            info!(
                "Loaded model `{}` with {} faces",
                self.name,
                mesh.face_count()
            );

            let mut model = Model::from_mesh(mesh)
                .with_name(mem::take(&mut self.name))
                .with_random_color();
            model.update_oob(&app.project.slice_config.platform_size);
            let result = PollResult::complete()
                .with_task(MeshManifold::new(&model))
                .with_task(BuildAccelerationStructures::new(&model));
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
