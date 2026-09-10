use clone_macro::clone;
use common::progress::Progress;
use slicer::mesh::MeshId;

use crate::{
    project::model::{MeshWarnings, Model},
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

pub struct MeshDefective {
    mesh_id: MeshId,
    progress: Progress,
    handle: TaskThread<bool>,
}

impl MeshDefective {
    pub fn new(mesh: &Model) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress, { mesh.mesh } as model], move || {
            model.is_defective(progress)
        }));

        Self {
            mesh_id: mesh.mesh.mesh_id(),
            progress,
            handle,
        }
    }
}

impl Task for MeshDefective {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Check for Mesh Defects")
            .into_poll_result(|result| {
                for model in app
                    .project
                    .models
                    .iter_mut()
                    .filter(|x| x.mesh.mesh_id() == self.mesh_id)
                {
                    model.warnings.set(MeshWarnings::Defective, !result);
                }
                PollResult::complete()
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Is Detective".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
