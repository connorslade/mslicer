use std::{fs::File, io::BufReader, mem, thread::JoinHandle};

use anyhow::Result;
use common::{
    progress::Progress,
    serde::{ReaderDeserializer, SliceDeserializer},
};
use mesh_format::load_mesh;

use slicer::mesh::Mesh;
use tracing::info;

use crate::{
    app::{
        App,
        project::model::Model,
        task::{
            MeshManifold, PollResult, Task, TaskStatus,
            acceleration_structures::BuildAccelerationStructures,
        },
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
    fn poll(&mut self, app: &mut App) -> PollResult {
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
                    return PollResult::complete();
                }
            };

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
            return result;
        }

        PollResult::pending()
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Loading Model".into(),
            details: Some(format!("Loading `{}`", self.name)),
            progress: self.progress.progress(),
        })
    }
}
