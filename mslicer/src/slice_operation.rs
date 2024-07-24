use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use common::misc::human_duration;
use goo_format::File as GooFile;
use image::RgbaImage;
use nalgebra::Vector2;
use parking_lot::{Condvar, MappedMutexGuard, Mutex, MutexGuard};
use slicer::slicer::Progress as SliceProgress;
use tracing::info;

#[derive(Clone)]
pub struct SliceOperation {
    start_time: Instant,
    completion: Arc<AtomicU32>,

    pub progress: SliceProgress,
    pub result: Arc<Mutex<Option<SliceResult>>>,

    pub preview_image: Arc<Mutex<Option<RgbaImage>>>,
    preview_condvar: Arc<Condvar>,
}

pub struct SliceResult {
    pub goo: GooFile,

    pub slice_preview_layer: usize,
    pub last_preview_layer: usize,
    pub preview_offset: Vector2<f32>,
    pub preview_scale: f32,
}

impl SliceOperation {
    pub fn new(progress: SliceProgress) -> Self {
        Self {
            start_time: Instant::now(),
            completion: Arc::new(AtomicU32::new(0)),
            progress,
            result: Arc::new(Mutex::new(None)),
            preview_image: Arc::new(Mutex::new(None)),
            preview_condvar: Arc::new(Condvar::new()),
        }
    }

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

    pub fn result(&self) -> MutexGuard<Option<SliceResult>> {
        self.result.lock()
    }
}
