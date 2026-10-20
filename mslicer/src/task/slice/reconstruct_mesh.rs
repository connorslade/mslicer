use clone_macro::clone;
use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};
use slicer::geometry::mesh::Mesh;
use tools::reconstruct_mesh;

use crate::task::{MeshLoad, PollResult, Task, TaskApp, TaskStatus, thread::TaskThread};

pub struct ReconstructMesh {
    progress: Progress,
    handle: TaskThread<Mesh>,
}

pub enum ReconstructionMode {
    MarchingCubes,
    GreedyRLE,
}

impl ReconstructMesh {
    pub fn new(
        config: SliceConfig,
        result: Vec<Layer>,
        mode: ReconstructionMode,
        subsample: u8,
    ) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            match mode {
                ReconstructionMode::MarchingCubes => {
                    reconstruct_mesh::marching_cubes(&progress, &config, &result, subsample)
                }
                ReconstructionMode::GreedyRLE => {
                    reconstruct_mesh::greedy_rle(&progress, &config, &result, subsample)
                }
            }
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

impl ReconstructionMode {
    pub const ALL: [Self; 2] = [Self::MarchingCubes, Self::GreedyRLE];

    pub fn name(&self) -> &str {
        match self {
            ReconstructionMode::MarchingCubes => "Marching Cubes",
            ReconstructionMode::GreedyRLE => "Greedy Meshing",
        }
    }
}
