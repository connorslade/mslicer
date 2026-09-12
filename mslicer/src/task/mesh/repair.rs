use clone_macro::clone;
use common::progress::Progress;
use slicer::mesh::Mesh;
use tools::repair;
use tracing::info;

use crate::{
    project::model::{Model, ModelId},
    task::{
        BuildAccelerationStructures, MeshDefective, PollResult, Task, TaskApp, TaskStatus,
        thread::TaskThread,
    },
};

pub struct MeshRepair {
    handle: TaskThread<Mesh>,
    progress: Progress,

    model: ModelId,
}

impl MeshRepair {
    pub fn new(model: &Model) -> Self {
        let mesh = model.mesh.clone();
        let half_edge = model.half_edge.clone().unwrap();
        let model = model.id;

        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            progress.set_total(1);
            let result = repair::MeshRepair {
                vertex_epsilon: 1e-4,
            }
            .repair(&mesh, half_edge);
            progress.set_finished();

            info!(
                "{{ unwelded_vertices: {}, holes: {} }}",
                result.unwelded_vertices, result.holes
            );
            result.mesh
        }));

        Self {
            handle,
            progress,
            model,
        }
    }
}

impl Task for MeshRepair {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Repair Mesh")
            .into_poll_result(|mesh| {
                let platform = app.project.slice_config.platform_size;
                let Some(model) = app.project.model(self.model) else {
                    return PollResult::complete();
                };

                model.replace_mesh(mesh, None, &platform);
                PollResult::complete()
                    .with_task(MeshDefective::new(model))
                    .with_task(BuildAccelerationStructures::new(model))
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Mesh Repair".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
