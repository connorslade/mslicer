use std::sync::Arc;

use clone_macro::clone;
use common::progress::Progress;
use slicer::geometry::{
    mesh::{HalfEdgeMesh, MeshId, MeshInner},
    partitioning::Bvh,
};

use crate::{
    core::project::model::Model,
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

pub struct BuildAccelerationStructures {
    mesh_id: MeshId,
    name: Option<String>,

    progress: Progress,
    handle: TaskThread<(Arc<Bvh>, Arc<HalfEdgeMesh>)>,
}

impl BuildAccelerationStructures {
    pub fn new(model: &Model) -> Self {
        let progress = Progress::new();
        let mesh = model.mesh.inner().clone();
        Self {
            mesh_id: model.mesh.mesh_id(),
            name: Some(model.name.clone()),
            handle: TaskThread::spawn(clone!([progress], move || {
                let bvh = Bvh::build(&mesh, progress);
                let half_edge = HalfEdgeMesh::build_from_mesh(&mesh);
                (Arc::new(bvh), Arc::new(half_edge))
            })),
            progress,
        }
    }

    pub fn new_mesh(mesh_id: MeshId, inner: Arc<MeshInner>) -> Self {
        let progress = Progress::new();
        Self {
            mesh_id,
            name: None,
            handle: TaskThread::spawn(clone!([progress], move || {
                let bvh = Bvh::build(&inner, progress);
                let half_edge = HalfEdgeMesh::build_from_mesh(&inner);
                (Arc::new(bvh), Arc::new(half_edge))
            })),
            progress,
        }
    }
}

impl Task for BuildAccelerationStructures {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        const FAILURE: &str = "Failed to Build Acceleration Structure";
        (self.handle.poll(app, FAILURE)).into_poll_result(|(bvh, half_edge)| {
            for model in app
                .project
                .models
                .iter_mut()
                .filter(|x| x.mesh.mesh_id() == self.mesh_id)
            {
                model.bvh = Some(bvh.clone());
                model.half_edge = Some(half_edge.clone());
            }
            PollResult::complete()
        })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Building Acceleration Structures".into(),
            details: self.name.as_ref().map(|name| format!("For `{name}`")),
            progress: self.progress.progress(),
        })
    }
}
