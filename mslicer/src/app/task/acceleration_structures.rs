use std::sync::Arc;

use clone_macro::clone;
use common::progress::Progress;
use slicer::{geometry::bvh::Bvh, half_edge::HalfEdgeMesh};

use crate::app::{
    App,
    project::model::Model,
    task::{PollResult, Task, TaskStatus, thread::TaskThread},
};

pub struct BuildAccelerationStructures {
    mesh: u32,
    name: String,

    progress: Progress,
    handle: TaskThread<(Arc<Bvh>, Arc<HalfEdgeMesh>)>,
}

impl BuildAccelerationStructures {
    pub fn new(model: &Model) -> Self {
        let progress = Progress::new();
        let mesh = model.mesh.inner().clone();
        Self {
            mesh: model.id,
            name: model.name.clone(),
            handle: TaskThread::spawn(clone!([progress], move || {
                let bvh = Bvh::build(&mesh, progress);
                let half_edge = HalfEdgeMesh::build(&mesh);
                (Arc::new(bvh), Arc::new(half_edge))
            })),
            progress,
        }
    }
}

impl Task for BuildAccelerationStructures {
    fn poll(&mut self, app: &mut App) -> PollResult {
        const FAILURE: &str = "Failed to Build Acceleration Structure";
        (self.handle.poll(app, FAILURE)).into_poll_result(|(bvh, half_edge)| {
            if let Some(model) = app.project.models.iter_mut().find(|x| x.id == self.mesh) {
                model.bvh = Some(bvh);
                model.half_edge = Some(half_edge);
            }
            PollResult::complete()
        })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Building Acceleration Structures".into(),
            details: Some(format!("For `{}`", self.name)),
            progress: self.progress.progress(),
        })
    }
}
