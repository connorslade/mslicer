use clone_macro::clone;
use common::progress::Progress;

use crate::app::{
    project::model::{MeshWarnings, Model},
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

pub struct MeshManifold {
    mesh: u32,
    progress: Progress,
    handle: TaskThread<bool>,
}

impl MeshManifold {
    pub fn new(mesh: &Model) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress, { mesh.mesh } as model], move || {
            model.is_manifold(progress)
        }));

        Self {
            mesh: mesh.id,
            progress,
            handle,
        }
    }
}

impl Task for MeshManifold {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Check Mesh Manifold")
            .into_poll_result(|result| {
                if let Some(model) = app.project.models.iter_mut().find(|x| x.id == self.mesh) {
                    model.warnings.set(MeshWarnings::NonManifold, !result);
                }
                PollResult::complete()
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Is Manifold".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
