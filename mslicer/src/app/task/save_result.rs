use std::sync::Arc;

use clone_macro::clone;
use common::{progress::Progress, serde::DynamicSerializer};
use slicer::format::FormatSliceFile;

use crate::app::task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread};

pub struct SaveResult {
    progress: Progress,
    file_name: String,
    handle: TaskThread<()>,
}

impl SaveResult {
    pub fn new(
        (file, file_name): (Arc<FormatSliceFile>, String),
        callback: impl FnOnce(Vec<u8>) + Send + 'static,
    ) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            let mut serializer = DynamicSerializer::new();
            file.serialize(&mut serializer, progress);
            callback(serializer.into_inner());
        }));
        SaveResult {
            progress,
            file_name,
            handle,
        }
    }
}

impl Task for SaveResult {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Write Slice Result")
            .into_poll_result(|_| PollResult::complete())
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Writing Slice Result".into(),
            details: Some(format!("Saving to {}", self.file_name)),
            progress: self.progress.progress(),
        })
    }
}
