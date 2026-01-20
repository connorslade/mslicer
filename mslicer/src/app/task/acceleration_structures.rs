use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use clone_macro::clone;
use common::progress::Progress;
use slicer::{geometry::bvh::Bvh, half_edge::HalfEdgeMesh};

use crate::app::{
    App,
    project::model::Model,
    task::{PollResult, Task, TaskStatus},
};

pub struct BuildAccelerationStructures {
    mesh: u32,
    name: String,

    progress: Progress,
    handle: Option<JoinHandle<(Arc<Bvh>, Arc<HalfEdgeMesh>)>>,
}

impl BuildAccelerationStructures {
    pub fn new(model: &Model) -> Self {
        let progress = Progress::new();

        let mesh = model.mesh.inner().clone();
        let handle = thread::spawn(clone!([progress], move || {
            let bvh = Bvh::build(&mesh, progress);
            let half_edge = HalfEdgeMesh::build(&mesh);
            (Arc::new(bvh), Arc::new(half_edge))
        }));

        Self {
            mesh: model.id,
            name: model.name.clone(),

            progress,
            handle: Some(handle),
        }
    }
}

impl Task for BuildAccelerationStructures {
    fn poll(&mut self, app: &mut App) -> PollResult {
        if self.progress.complete() {
            let (bvh, half_edge) = self.handle.take().unwrap().join().unwrap();
            if let Some(model) = app.project.models.iter_mut().find(|x| x.id == self.mesh) {
                model.bvh = Some(bvh);
                model.half_edge = Some(half_edge);
            }

            return PollResult::complete();
        }

        PollResult::pending()
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Building Acceleration Structures".into(),
            details: Some(format!("For `{}`", self.name)),
            progress: self.progress.progress(),
        })
    }
}
