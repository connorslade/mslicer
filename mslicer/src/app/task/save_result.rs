use std::{sync::Arc, thread};

use clone_macro::clone;
use common::{progress::Progress, serde::DynamicSerializer};
use slicer::format::FormatSliceFile;

use crate::app::{
    App,
    task::{PollResult, Task, TaskStatus},
};

pub struct SaveResult {
    progress: Progress,
    file_name: String,
}

impl SaveResult {
    pub fn new(
        (file, file_name): (Arc<FormatSliceFile>, String),
        callback: impl FnOnce(Vec<u8>) + Send + 'static,
    ) -> Self {
        let progress = Progress::new();
        thread::spawn(clone!([progress], move || {
            let mut serializer = DynamicSerializer::new();
            file.serialize(&mut serializer, progress);
            callback(serializer.into_inner());
        }));
        SaveResult {
            progress,
            file_name,
        }
    }
}

impl Task for SaveResult {
    fn poll(&mut self, _app: &mut App) -> PollResult {
        PollResult::from_bool(self.progress.complete())
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Writing Slice Result".into(),
            details: Some(format!("Saving to {}", self.file_name)),
            progress: self.progress.progress(),
        })
    }
}
