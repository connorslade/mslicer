use std::{
    collections::HashMap,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use common::{
    container::Run,
    misc::human_duration,
    progress::{CombinedProgress, Progress},
    slice::{DynSlicedFile, Layer, SliceConfig, VectorLayer, format::Format},
    units::{Miliseconds, Milliliters, Seconds},
};
use egui::Color32;
use image::RgbaImage;
use parking_lot::{Condvar, Mutex, MutexGuard};
use slicer::{slicer::slice_vector::SvgFile, util};
use tracing::info;

#[derive(Clone)]
pub struct SliceOperation {
    inner: Arc<SliceOperationInner>,
}

pub struct SliceOperationInner {
    start_time: Instant,
    pub progress: Progress,
    pub post_processing_progress: CombinedProgress<1>,
    pub result: Mutex<Option<SliceResult>>,

    preview_image: Mutex<Option<Arc<RgbaImage>>>,
    preview_condvar: Condvar,
}

pub struct SliceResult {
    pub config: SliceConfig,
    pub elapsed: Duration,
    pub fresh: bool,

    pub inner: GenericSliceResult,
}

pub enum GenericSliceResult {
    Raster(RasterSliceResult),
    Vector(VectorSliceResult),
}

#[derive(Clone)]
pub enum GenericSliceData {
    Raster { data: Arc<Vec<Layer>>, voxels: u64 },
    Vector { data: Arc<Vec<VectorLayer>> },
}

pub struct RasterSliceResult {
    pub layers: Arc<Vec<Layer>>,
    pub annotations: Arc<Annotations>,
    pub detected_islands: bool,

    pub voxels: u64,
    pub volume: Milliliters,
    pub print_time: Seconds,
}

pub struct VectorSliceResult {
    pub layers: Arc<Vec<VectorLayer>>,
}

pub const ISLAND_COLOR: Color32 = Color32::from_rgb(159, 44, 54);

#[derive(Default)]
pub struct Annotations {
    layers: Mutex<HashMap<usize, Vec<Run<Annotation>>>>,
    updated: AtomicBool,
}

pub struct LockedAnnotations<'a> {
    layers: MutexGuard<'a, HashMap<usize, Vec<Run<Annotation>>>>,
    updated: &'a AtomicBool,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Annotation {
    None = 0b00,
    Island = 0b01,
}

impl SliceOperation {
    pub fn new(slice: Progress, post_process: CombinedProgress<1>) -> Self {
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
        self.preview_image.lock().replace(Arc::new(image));
        self.preview_condvar.notify_all();
    }

    pub fn preview_image(&self) -> Arc<RgbaImage> {
        let mut preview_image = self.preview_image.lock();
        while preview_image.is_none() {
            self.preview_condvar.wait(&mut preview_image);
        }

        preview_image.as_ref().unwrap().clone()
    }

    pub fn add_raster_result(&self, config: SliceConfig, layers: Vec<Layer>) {
        let voxels = (layers.iter())
            .map(|x| (x.data.iter().filter(|x| x.value != 0).map(|x| x.length)).sum::<u64>())
            .sum::<u64>();

        let elapsed = self.start_time.elapsed();
        info!("Raster slice operation completed in {:?}", elapsed);

        let raster = RasterSliceResult {
            voxels,
            volume: (voxels as f32 * config.voxel_volume()).convert(),
            print_time: config.print_time(layers.len() as u32),

            layers: Arc::new(layers),
            annotations: Arc::new(Annotations::default()),
            detected_islands: false,
        };

        self.result().replace(SliceResult {
            config,
            elapsed,
            fresh: true,
            inner: raster.into(),
        });
    }

    pub fn add_vector_result(&self, config: SliceConfig, layers: Arc<Vec<VectorLayer>>) {
        let elapsed = self.start_time.elapsed();
        info!("Vector slice operation completed in {:?}", elapsed);

        self.result().replace(SliceResult {
            config,
            elapsed,
            fresh: true,
            inner: VectorSliceResult { layers }.into(),
        });
    }

    pub fn result(&self) -> MutexGuard<'_, Option<SliceResult>> {
        self.result.lock()
    }
}

impl Annotations {
    pub fn lock(&self) -> LockedAnnotations<'_> {
        LockedAnnotations {
            layers: self.layers.lock(),
            updated: &self.updated,
        }
    }

    pub fn take_updated(&self) -> bool {
        self.updated.swap(false, Ordering::Relaxed)
    }
}

impl<'a> LockedAnnotations<'a> {
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
        self.updated.store(true, Ordering::Relaxed);
    }
}

impl SliceResult {
    pub fn completion(&self) -> String {
        let time = self.elapsed.as_millis() as f32;
        human_duration(Miliseconds::new(time))
    }

    /// Assumes result is not None
    pub fn slice_data(&self) -> GenericSliceData {
        match &self.inner {
            GenericSliceResult::Raster(result) => GenericSliceData::Raster {
                data: result.layers.clone(),
                voxels: result.voxels,
            },
            GenericSliceResult::Vector(result) => GenericSliceData::Vector {
                data: result.layers.clone(),
            },
        }
    }
}

impl GenericSliceResult {
    pub fn as_raster(&self) -> Option<&RasterSliceResult> {
        match self {
            GenericSliceResult::Raster(raster) => Some(raster),
            _ => None,
        }
    }

    pub fn layers(&self) -> usize {
        match self {
            GenericSliceResult::Raster(raster) => raster.layers.len(),
            GenericSliceResult::Vector(vector) => vector.layers.len(),
        }
    }
}

impl GenericSliceData {
    pub fn file(
        &self,
        config: &SliceConfig,
        preview_image: &RgbaImage,
        format: Format,
    ) -> DynSlicedFile {
        match &self {
            GenericSliceData::Raster { data, voxels } => {
                let format = format.as_raster().unwrap();
                let mut file = util::export_raster(config, data.iter(), *voxels, format);
                file.set_preview(preview_image);
                file
            }
            GenericSliceData::Vector { data } => {
                let platform = config.platform_resolution.xy();
                let file = SvgFile::new(platform, data.clone());
                Box::new(file)
            }
        }
    }
}

impl Deref for SliceOperation {
    type Target = SliceOperationInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<RasterSliceResult> for GenericSliceResult {
    fn from(value: RasterSliceResult) -> Self {
        Self::Raster(value)
    }
}

impl From<VectorSliceResult> for GenericSliceResult {
    fn from(value: VectorSliceResult) -> Self {
        Self::Vector(value)
    }
}
