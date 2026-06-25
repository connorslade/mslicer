use clone_macro::clone;
use common::{geometry::convex_hull, progress::Progress, slice::SliceConfig, units::Milimeter};
use nalgebra::{Vector2, Vector3};
use slicer::mesh::Mesh;
use tools::auto_layout;

use crate::{
    project::model::Model,
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

pub struct AutoLayout {
    handle: TaskThread<Vec<(u32, Vector3<f32>)>>,
    progress: Progress,
}

impl AutoLayout {
    pub fn new(
        slice_config: &SliceConfig,
        models: &[Model],
        (padding, segment_steps): (f32, f32),
    ) -> Self {
        let platform = (slice_config.platform_size.xy()).map(|x| x.get::<Milimeter>());
        let models = (models.iter().filter(|x| !x.hidden))
            .map(|x| {
                let points = project_down(&x.mesh);
                auto_layout::Model::new(x.id, x.mesh.position(), convex_hull(&points))
            })
            .collect::<Vec<_>>();

        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            auto_layout::AutoLayoutNFP::new(platform, models)
                .padding(padding)
                .segment_steps(segment_steps)
                .layout(false, progress)
                .unwrap()
                .1
        }));

        Self { handle, progress }
    }
}

impl Task for AutoLayout {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Layout Models")
            .into_poll_result(|x| {
                for (model, new_pos) in x.iter() {
                    if let Some(model) = app.project.models.iter_mut().find(|x| x.id == *model) {
                        model.mesh.set_position(*new_pos);
                    }
                }

                PollResult::complete()
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Auto Layout".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}

fn project_down(mesh: &Mesh) -> Vec<Vector2<f32>> {
    mesh.vertices()
        .iter()
        .map(|x| mesh.transform(&x).xy())
        .collect::<Vec<_>>()
}
