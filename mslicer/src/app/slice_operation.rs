use std::{
    collections::HashMap,
    ops::Deref,
    sync::Arc,
    time::{Duration, Instant},
};

use common::{
    container::Run,
    misc::human_duration,
    progress::{CombinedProgress, Progress},
    slice::{DynSlicedFile, SliceConfig},
    units::{Miliseconds, Milliliters, Seconds},
};
use egui::Color32;
use image::RgbaImage;
use nalgebra::Vector2;
use parking_lot::{Condvar, MappedMutexGuard, Mutex, MutexGuard};
use tracing::info;

#[derive(Clone)]
pub struct SliceOperation {
    inner: Arc<SliceOperationInner>,
}

pub struct SliceOperationInner {
    start_time: Instant,
    pub progress: Progress,
    pub post_processing_progress: CombinedProgress<2>,
    pub result: Mutex<Option<SliceResult>>,

    preview_image: Mutex<Option<RgbaImage>>,
    preview_condvar: Condvar,
}

pub struct SliceResult {
    pub file: Arc<DynSlicedFile>,
    pub elapsed: Duration,
    pub annotations: Arc<Mutex<Annotations>>,
    pub volume: Milliliters,
    pub print_time: Seconds,

    pub detected_islands: bool,
    pub slice_preview_layer: usize,
    pub last_preview_layer: usize,
    pub preview_offset: Vector2<f32>,
    pub preview_scale: f32,
    pub layer_count: (usize, u8),
}

pub const ISLAND_COLOR: Color32 = Color32::from_rgb(159, 44, 54);

#[derive(Default)]
pub struct Annotations {
    layers: HashMap<usize, Vec<Run<Annotation>>>,
}

#[derive(Clone, Copy)]
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

    pub fn add_result(&self, config: &SliceConfig, (file, voxels): (DynSlicedFile, u64)) {
        let elapsed = self.start_time.elapsed();
        info!("Slice operation completed in {:?}", elapsed);

        let layers = file.info().layers as usize;
        let volume = (voxels as f32 * config.voxel_volume()).convert();
        let print_time = config.print_time(layers as u32);

        self.result.lock().replace(SliceResult {
            elapsed,
            file: Arc::new(file),
            annotations: Arc::new(Mutex::new(Annotations::default())),
            volume,
            print_time,

            detected_islands: false,
            slice_preview_layer: 0,
            last_preview_layer: 0,
            preview_offset: Vector2::new(0.0, 0.0),
            preview_scale: 1.0,
            layer_count: (layers, layers.to_string().len() as u8),
        });
    }

    pub fn result(&self) -> MutexGuard<'_, Option<SliceResult>> {
        self.result.lock()
    }
}

impl Annotations {
    pub fn contains(&self, layer: usize) -> bool {
        if let Some(layer) = self.layers.get(&layer) {
            layer.iter().any(|x| !matches!(x.value, Annotation::None))
        } else {
            false
        }
    }

    pub fn decode_layer(&self, layer: usize, buffer: &mut [u8]) {
        let Some(layer) = self.layers.get(&layer) else {
            return;
        };

        let mut pos = 0;
        for run in layer.iter() {
            let len = run.length as usize;
            buffer[pos..(pos + len)].fill(run.value as u8);
            pos += len;
        }
    }

    pub fn insert_layer(&mut self, annotation: Annotation, layer: usize, runs: &[u64]) {
        assert!(!self.layers.contains_key(&layer)); // not yet implemented!

        let runs = runs
            .iter()
            .enumerate()
            .map(|(i, &l)| Run {
                length: l,
                value: [Annotation::None, annotation][(i % 2 != 0) as usize],
            })
            .collect::<Vec<_>>();
        self.layers.insert(layer, runs);
    }
}

impl SliceResult {
    pub fn completion(&self) -> String {
        let time = self.elapsed.as_millis() as f32;
        human_duration(Miliseconds::new(time))
    }
}

impl Deref for SliceOperation {
    type Target = SliceOperationInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
