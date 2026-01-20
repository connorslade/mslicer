use std::{
    mem,
    thread::{self, JoinHandle},
};

use clone_macro::clone;
use common::progress::Progress;

use crate::app::{
    App,
    project::model::{MeshWarnings, Model},
    task::{PollResult, Task, TaskStatus},
};

pub struct MeshManifold {
    mesh: u32,
    progress: Progress,
    handle: Option<JoinHandle<bool>>,
}

impl MeshManifold {
    pub fn new(mesh: &Model) -> Self {
        let progress = Progress::new();
        let handle = thread::spawn(clone!([progress, { mesh.mesh } as model], move || {
            model.is_manifold(progress)
        }));

        Self {
            mesh: mesh.id,
            progress,
            handle: Some(handle),
        }
    }
}

impl Task for MeshManifold {
    fn poll(&mut self, app: &mut App) -> PollResult {
        if self.progress.complete() {
            let result = mem::take(&mut self.handle).unwrap().join().unwrap();
            if let Some(model) = app.project.models.iter_mut().find(|x| x.id == self.mesh) {
                model.warnings.set(MeshWarnings::NonManifold, !result);
            }

            PollResult::complete()
        } else {
            PollResult::pending()
        }
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Is Manifold".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
