use clone_macro::clone;
use common::progress::Progress;
use slicer::mesh::Mesh;

use crate::{
    project::model::{Model, ModelId},
    task::{
        BuildAccelerationStructures, PollResult, Task, TaskApp, TaskStatus, thread::TaskThread,
    },
};

pub struct FlipWinding {
    progress: Progress,
    handle: TaskThread<Mesh>,

    model: ModelId,
}

impl FlipWinding {
    pub fn new(model: &Model) -> Self {
        let progress = Progress::new();
        let mesh = model.mesh.inner().clone();
        let model = model.id;

        let handle = TaskThread::spawn(clone!([progress], move || {
            progress.set_total(mesh.faces.len() as u64);
            let mut flipped = Vec::with_capacity(mesh.faces.len());
            for [a, b, c] in mesh.faces.iter() {
                flipped.push([*a, *c, *b]);
                progress.add_complete(1);
            }

            progress.set_finished();
            Mesh::new_boxed(mesh.vertices.clone(), flipped.into_boxed_slice())
        }));

        Self {
            progress,
            handle,
            model,
        }
    }
}

impl Task for FlipWinding {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Flip Winding Order")
            .into_poll_result(|mesh| {
                let platform = app.project.slice_config.platform_size;
                let Some(model) = app.project.model(self.model) else {
                    return PollResult::complete();
                };

                model.replace_mesh(mesh, None, &platform);
                PollResult::complete().with_task(BuildAccelerationStructures::new(model))
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Flipping Winding Order".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
