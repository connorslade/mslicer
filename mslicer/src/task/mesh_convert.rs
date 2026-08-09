use std::sync::Arc;

use clone_macro::clone;
use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};
use slicer::{mesh::Mesh, post_process::mesh_convert::mesh_convert};

use crate::task::{MeshLoad, PollResult, Task, TaskApp, TaskStatus, thread::TaskThread};

pub struct MeshConvert {
    progress: Progress,
    handle: TaskThread<Mesh>,
}

impl MeshConvert {
    pub fn new(config: SliceConfig, result: Arc<Vec<Layer>>) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            mesh_convert(&progress, &config, &result)
        }));

        Self { progress, handle }
    }
}

impl Task for MeshConvert {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Unexpected Error while Converting to Mesh")
            .into_poll_result(|mesh| {
                PollResult::complete()
                    .with_task(MeshLoad::complete("Reconstructed Model".into(), mesh))
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Converting to Mesh".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
