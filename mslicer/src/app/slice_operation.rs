use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use common::{
    container::Run,
    misc::human_duration,
    progress::{CombinedProgress, Progress},
};
use image::RgbaImage;
use nalgebra::Vector2;
use parking_lot::{Condvar, MappedMutexGuard, Mutex, MutexGuard};
use slicer::format::FormatSliceFile;
use tracing::info;

#[derive(Clone)]
pub struct SliceOperation {
    inner: Arc<SliceOperationInner>,
}

pub struct SliceOperationInner {
    start_time: Instant,
    completion: AtomicU32,

    pub progress: Progress,
    pub post_processing_progress: CombinedProgress<2>,
    pub result: Mutex<Option<SliceResult>>,

    preview_image: Mutex<Option<RgbaImage>>,
    preview_condvar: Condvar,
}

pub struct SliceResult {
    pub file: Arc<FormatSliceFile>,
    pub annotations: Arc<Mutex<Vec<Vec<Run>>>>,

    pub slice_preview_layer: usize,
    pub last_preview_layer: usize,
    pub preview_offset: Vector2<f32>,
    pub preview_scale: f32,
    pub layer_count: (usize, u8),
}

#[repr(u8)]
pub enum Annotation {
    None = 0b00,
    Island = 0b01,
}

impl SliceOperation {
    pub fn new(slice: Progress, post_process: CombinedProgress<2>) -> Self {
        Self {
            inner: Arc::new(SliceOperationInner {
                start_time: Instant::now(),
                completion: AtomicU32::new(0),

                progress: slice,
                post_processing_progress: post_process,
                result: Mutex::new(None),

                preview_image: Mutex::new(None),
                preview_condvar: Condvar::new(),
            }),
        }
    }
}

impl SliceOperationInner {
    pub fn completion(&self) -> Option<String> {
        let time = self.completion.load(Ordering::Relaxed);
        (time != 0).then(|| human_duration(Duration::from_millis(time as u64)))
    }

    pub fn needs_preview_image(&self) -> bool {
        self.preview_image.lock().is_none()
    }

    pub fn add_preview_image(&self, image: RgbaImage) {
        self.preview_image.lock().replace(image);
        self.preview_condvar.notify_all();
    }

    pub fn preview_image(&self) -> MappedMutexGuard<'_, RgbaImage> {
        let mut preview_image = self.preview_image.lock();
        while preview_image.is_none() {
            self.preview_condvar.wait(&mut preview_image);
        }

        MutexGuard::map(preview_image, |image| image.as_mut().unwrap())
    }

    pub fn add_result(&self, result: SliceResult) {
        let elapsed = self.start_time.elapsed();
        self.completion
            .store(elapsed.as_millis() as u32, Ordering::Relaxed);

        info!("Slice operation completed in {:?}", elapsed);
        self.result.lock().replace(result);
    }

    pub fn result(&self) -> MutexGuard<'_, Option<SliceResult>> {
        self.result.lock()
    }
}

impl Deref for SliceOperation {
    type Target = SliceOperationInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
