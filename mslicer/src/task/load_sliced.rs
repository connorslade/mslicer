use std::{fs, path::PathBuf};

use clone_macro::clone;
use common::{
    progress::{CombinedProgress, Progress},
    slice::{Layer, SliceConfig, format::RasterFormat},
};
use image::RgbaImage;

use crate::{
    app::{SLICE_PREVIEW_SIZE, slice_operation::SliceOperation},
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
    windows::Tab,
};

pub struct LoadSliced {
    progress: Progress,
    handle: TaskThread<(SliceConfig, Vec<Layer>, Vec<RgbaImage>)>,
}

impl LoadSliced {
    pub fn new(path: PathBuf) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            let ext = path.extension().unwrap().to_string_lossy();
            let format = RasterFormat::from_extension(&ext).unwrap();

            let data = fs::read(path).unwrap(); // todo:handle
            slicer::util::load_sliced(&progress, &format, &data).unwrap() // todo: handle
        }));
        Self { progress, handle }
    }
}

impl Task for LoadSliced {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Load Sliced File")
            .into_poll_result(|(config, layers, image)| {
                let operation = SliceOperation::new(
                    Progress::already_complete(),
                    CombinedProgress::already_complete(),
                );

                operation.add_raster_result(config, layers);
                image.into_iter().for_each(|x| operation.add_preview(x));
                operation.set_loaded();

                app.slice_operation.replace(operation);
                app.panels.focus_tab(Tab::Sliced, SLICE_PREVIEW_SIZE);

                PollResult::complete()
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Load Sliced".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
