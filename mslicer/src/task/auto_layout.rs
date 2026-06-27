use clone_macro::clone;
use common::{progress::Progress, slice::SliceConfig, units::Milimeter};
use tools::auto_layout::{self, Placement};

use crate::{
    project::model::Model,
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
    windows::tools::auto_layout::{apply_placement, layout_cache},
};

pub struct AutoLayout {
    handle: TaskThread<Vec<Placement>>,
    progress: Progress,
}

impl AutoLayout {
    pub fn new(
        slice_config: &SliceConfig,
        models: &[Model],
        (padding, segment_steps): (f32, f32),
    ) -> Self {
        let platform = (slice_config.platform_size.xy()).map(|x| x.get::<Milimeter>());
        let (mut cache, models) = layout_cache(padding, models);

        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            auto_layout::AutoLayoutNfp::new(platform, models, &mut cache)
                .segment_steps(segment_steps)
                .layout(progress)
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
                x.iter()
                    .for_each(|x| apply_placement(&mut app.project.models, x));
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
