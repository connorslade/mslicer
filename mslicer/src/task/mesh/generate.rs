use std::{borrow::Cow, mem};

use clone_macro::clone;
use common::progress::Progress;
use slicer::mesh::Mesh;

use crate::task::{MeshLoad, PollResult, Task, TaskApp, TaskStatus, thread::TaskThread};

pub struct GenerateMesh {
    name: Cow<'static, str>,
    progress: Progress,

    join: TaskThread<Mesh>,
}

impl GenerateMesh {
    pub fn new(
        name: Cow<'static, str>,
        generator: impl FnOnce(&Progress) -> Mesh + Send + 'static,
    ) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || { generator(&progress) }));

        Self {
            name,
            progress,
            join: handle,
        }
    }
}

impl Task for GenerateMesh {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.join
            .poll(app, &format!("Failed to Generate {} Mesh", self.name))
            .into_poll_result(|result| {
                let name = mem::take(&mut self.name).into_owned();
                PollResult::complete().with_task(MeshLoad::complete(name, result))
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: format!("Generating {}", self.name).into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
