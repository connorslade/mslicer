use std::{fs::File, io::BufReader, mem, path::PathBuf};

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
    file_path: Option<PathBuf>,
    model_index: Option<usize>,
}

impl MeshLoad {
    pub fn file(path: PathBuf, name: String, format: String) -> Self {
        let file = File::open(&path).unwrap();
        let des = ReaderDeserializer::new(BufReader::new(file));
        let progress = Progress::new();
        Self {
            join: TaskThread::spawn(clone!([progress], move || {
                load_mesh(des, &format, progress).unwrap()
            })),
            progress,
            name,
            file_path: Some(path),
            model_index: None,
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
            file_path: None,
            model_index: None,
        }
    }

    pub fn reload(file_path: PathBuf, model_index: usize) -> Self {
        let file = File::open(&file_path).unwrap();
        let des = ReaderDeserializer::new(BufReader::new(file));
        let progress = Progress::new();
        Self {
            join: TaskThread::spawn(clone!([progress, file_path], move || {
                load_mesh(
                    des,
                    &file_path.extension().unwrap_or_default().to_string_lossy(),
                    progress,
                )
                .unwrap()
            })),
            progress,
            name: file_path.to_string_lossy().into_owned(),
            file_path: Some(file_path),
            model_index: Some(model_index),
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

            if let Some(model_index) = self.model_index {
                // Reload existing model
                let model = &mut app.project.models[model_index];
                model.replace_mesh(mesh, &app.project.slice_config.platform_size);

                PollResult::complete()
                    .with_task(MeshManifold::new(model))
                    .with_task(BuildAccelerationStructures::new(model))
            } else {
                // Create new model
                let mut model = Model::from_mesh(mesh)
                    .with_name(mem::take(&mut self.name))
                    .with_random_color();

                if let Some(file_path) = self.file_path.take() {
                    model = model.with_file_path(file_path);
                }

                model.update_oob(&app.project.slice_config.platform_size);
                let result = PollResult::complete()
                    .with_task(MeshManifold::new(&model))
                    .with_task(BuildAccelerationStructures::new(&model));
                app.project.models.push(model);
                result
            }
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
