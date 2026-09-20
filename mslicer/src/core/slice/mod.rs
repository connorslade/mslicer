use std::{iter, ops::Deref, sync::Arc, time::Instant};

use common::{
    misc::IteratorExt,
    progress::{CombinedProgress, Progress},
    slice::{Layer, SliceConfig, VectorLayer, print_time},
    units::{CubicMilimeters, Milimeters},
};
use image::RgbaImage;
use itertools::Itertools;
use parking_lot::{Mutex, MutexGuard};
use tracing::info;

use crate::{
    core::slice::{
        annotations::Annotations,
        result::{RasterSliceResult, SliceResult, VectorSliceResult},
    },
    util::management::LazyTextureId,
};

pub mod annotations;
pub mod result;
pub mod runner;

#[derive(Clone)]
pub struct SliceOperation {
    inner: Arc<SliceOperationInner>,
}

pub struct SliceOperationInner {
    start_time: Instant,
    pub progress: Progress,
    pub post_processing_progress: CombinedProgress<2>,
    pub result: Mutex<Option<SliceResult>>,
    pub previews: Mutex<Option<PreviewImage>>,
}

pub struct PreviewImage {
    pub image: Arc<RgbaImage>,
    pub texture: LazyTextureId,
}

impl SliceOperation {
    pub fn new(slice: Progress, post_process: CombinedProgress<2>) -> Self {
        Self {
            inner: Arc::new(SliceOperationInner {
                start_time: Instant::now(),
                progress: slice,
                post_processing_progress: post_process,
                result: Mutex::new(None),
                previews: Mutex::new(Default::default()),
            }),
        }
    }
}

impl SliceOperationInner {
    pub fn needs_previews(&self) -> bool {
        self.previews.lock().is_none()
    }

    pub fn add_preview(&self, image: RgbaImage) {
        *self.previews.lock() = Some(PreviewImage {
            image: Arc::new(image),
            texture: LazyTextureId::empty(),
        });
    }

    pub fn preview(&self) -> Arc<RgbaImage> {
        self.previews.lock().as_ref().unwrap().image.clone()
    }

    pub fn add_raster_result(&self, config: SliceConfig, layers: Vec<Layer>) {
        let heights = iter::once(Milimeters::new(0.0))
            .chain(layers.iter().map(|x| x.height))
            .tuple_windows()
            .map(|(a, b)| b - a);
        let volume = (layers.iter().zip(heights))
            .map(|(l, h)| l.area as f32 * config.pixel_area() * h)
            .fold(CubicMilimeters::new(0.0), |a, b| a + b)
            .convert();

        let voxels = (layers.iter())
            .flat_map(|x| x.data.iter().filter(|x| x.value != 0).map(|x| x.length))
            .sum::<u64>();

        let elapsed = self.start_time.elapsed();
        info!("Raster slice operation completed in {:?}", elapsed);

        let variable_layer_height = !(layers.iter())
            .map(|x| x.height.raw())
            .tuple_windows()
            .map(|(a, b)| b - a)
            .all_equal_float(0.001);

        let raster = RasterSliceResult {
            voxels,
            volume,
            print_time: print_time(layers.iter()),

            layers,
            annotations: Arc::new(Annotations::default()),
            detected_islands: false,
        };

        self.result().replace(SliceResult {
            config,
            elapsed,
            fresh: true,
            sliced: true,

            variable_layer_height,
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
            sliced: true,

            variable_layer_height: false,
            inner: VectorSliceResult { layers }.into(),
        });
    }

    pub fn set_loaded(&self) {
        self.result().as_mut().unwrap().sliced = false;
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
