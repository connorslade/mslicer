use clone_macro::clone;
use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};
use slicer::mesh::Mesh;
use tools::reconstruct_mesh::marching_cubes;

use crate::task::{MeshLoad, PollResult, Task, TaskApp, TaskStatus, thread::TaskThread};

pub struct ReconstructMesh {
    progress: Progress,
    handle: TaskThread<Mesh>,
}

impl ReconstructMesh {
    pub fn new(config: SliceConfig, result: Vec<Layer>, subsample: u8) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            marching_cubes(&progress, &config, &result, subsample)
        }));

        Self { progress, handle }
    }
}

impl Task for ReconstructMesh {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Unexpected Error while Reconstructing Mesh")
            .into_poll_result(|mesh| {
                PollResult::complete()
                    .with_task(MeshLoad::complete("Reconstructed Model".into(), mesh))
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Reconstructing Mesh".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
