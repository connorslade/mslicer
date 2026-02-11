use std::sync::Arc;

use clone_macro::clone;
use common::progress::Progress;
use slicer::{format::FormatSliceFile, post_process::island_detection::detect_islands};

use crate::app::task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread};

pub struct IslandDetection {
    progress: Progress,
    handle: TaskThread<()>,
}

impl IslandDetection {
    pub fn new(file: Arc<FormatSliceFile>) -> Self {
        let progress = Progress::new();
        Self {
            handle: TaskThread::spawn(clone!([file, progress], move || {
                detect_islands(&file, progress);
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
