use std::{fs::File, path::PathBuf, sync::Arc};

use clone_macro::clone;
use common::{
    progress::{CombinedProgress, Progress},
    serde::DynamicSerializer,
    slice::{
        Layer, SliceConfig,
        format::{Format, RasterFormat},
    },
};
use image::RgbaImage;

use crate::{
    app::{
        SLICE_PREVIEW_SIZE,
        slice_operation::{GenericSliceData, SliceOperation},
    },
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
    windows::Tab,
};

pub struct SaveSliced {
    progress: CombinedProgress<2>,
    file_name: String,
    handle: TaskThread<()>,
}

impl SaveSliced {
    pub fn new(
        (format, file, config, preview): (Format, GenericSliceData, SliceConfig, Arc<RgbaImage>),
        file_name: String,
        callback: impl FnOnce(Vec<u8>) + Send + 'static,
    ) -> Self {
        let progress = CombinedProgress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            let file = file.file(&progress[0], &config, &preview, format);

            let mut serializer = DynamicSerializer::new();
            file.serialize(&mut serializer, &progress[1]);
            callback(serializer.into_inner());
        }));
        SaveSliced {
            progress,
            file_name,
            handle,
        }
    }
}

impl Task for SaveSliced {
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

pub struct LoadSliced {
    progress: Progress,
    operation: Option<SliceOperation>,
    handle: TaskThread<(SliceConfig, Vec<Layer>, Vec<RgbaImage>)>,
}

impl LoadSliced {
    pub fn new(path: PathBuf) -> Self {
        let progress = Progress::new();
        let operation = SliceOperation::new(
            Progress::already_complete(),
            CombinedProgress::already_complete(),
        );

        let handle = TaskThread::spawn(clone!([progress], move || {
            let ext = path.extension().unwrap().to_string_lossy();
            let format = RasterFormat::from_extension(&ext).unwrap();

            let file = File::open(path).unwrap();
            let file = slicer::util::load_sliced(&format, file).unwrap();
            (file.slice_config(), file.layers(&progress), file.previews())
        }));

        Self {
            progress,
            operation: Some(operation),
            handle,
        }
    }
}

impl Task for LoadSliced {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Load Sliced File")
            .into_poll_result(|(config, layers, image)| {
                let operation = self.operation.take().unwrap();
                operation.add_raster_result(config, layers);
                (image.into_iter()).for_each(|x| operation.add_preview(x));
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
