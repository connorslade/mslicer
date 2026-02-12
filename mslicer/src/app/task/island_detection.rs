use std::sync::Arc;

use clone_macro::clone;
use common::{container::Run, progress::Progress};
use parking_lot::Mutex;
use slicer::{format::FormatSliceFile, post_process::island_detection::detect_islands};

use crate::app::task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread};

pub struct IslandDetection {
    progress: Progress,
    handle: TaskThread<()>,
}

impl IslandDetection {
    pub fn new(file: Arc<FormatSliceFile>, annotations: Arc<Mutex<Vec<Vec<Run>>>>) -> Self {
        let progress = Progress::new();
        Self {
            handle: TaskThread::spawn(clone!([file, progress], move || {
                let islands = detect_islands(&file, progress);
                *annotations.lock() = islands
                    .into_iter()
                    .map(|layer| {
                        layer
                            .into_iter()
                            .enumerate()
                            .map(|(i, l)| Run {
                                length: l as u64,
                                value: if i % 2 == 0 { 0 } else { 1 },
                            })
                            .collect()
                    })
                    .collect();
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
