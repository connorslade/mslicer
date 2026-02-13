use std::sync::Arc;

use clone_macro::clone;
use common::{progress::Progress, slice::DynSlicedFile};
use parking_lot::Mutex;
use slicer::post_process::island_detection::detect_islands;

use crate::app::{
    slice_operation::{Annotation, Annotations},
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

pub struct IslandDetection {
    progress: Progress,
    handle: TaskThread<()>,
}

impl IslandDetection {
    pub fn new(file: Arc<DynSlicedFile>, annotations: Arc<Mutex<Annotations>>) -> Self {
        let progress = Progress::new();
        Self {
            handle: TaskThread::spawn(clone!([file, progress], move || {
                let islands = detect_islands(&file, progress, true);

                let mut annotations = annotations.lock();
                for (layer, runs) in islands
                    .into_iter()
                    .enumerate()
                    .filter(|(_, runs)| runs.len() > 1)
                {
                    annotations.insert_layer(Annotation::Island, layer + 1, &runs);
                }
            })),
            progress,
        }
    }
}

impl Task for IslandDetection {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        (self.handle.poll(app, "...")).into_poll_result(|_| PollResult::complete())
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Detecting Islands".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
