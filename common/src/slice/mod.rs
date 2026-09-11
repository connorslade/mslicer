//! Simplified configuration for slicing a model.

use std::sync::Arc;

use image::RgbaImage;
use nalgebra::{Vector2, Vector3};

mod config;
pub mod format;
pub use config::{ExposureConfig, ExposureRemap, SliceConfig, Supersample};
pub use format::SliceMode;

use crate::{
    container::Run,
    progress::Progress,
    serde::DynamicSerializer,
    units::{Milimeters, Seconds},
};

/// Boxed [`SlicedFile`].
pub type DynSlicedFile = Box<dyn SlicedFile + Send + Sync>;

/// Sliced file interface.
///
/// Implemented by all format File types.
pub trait SlicedFile {
    fn serialize(&self, ser: &mut DynamicSerializer, progress: &Progress);
    fn set_preview(&mut self, preview: &RgbaImage);
    fn slice_config(&self) -> SliceConfig;
    fn layers(&self, progress: &Progress) -> Vec<Layer>;
    fn previews(&self) -> Vec<RgbaImage>;
}

/// Layer encoder interface.
///
/// Implemented by all format layer encoders.
pub trait EncodableLayer {
    type Output: Send;

    fn new(platform: Vector2<u32>) -> Self;
    fn add_run(&mut self, length: u64, value: u8);
    fn finish(
        self,
        config: &SliceConfig,
        exposure: &ExposureConfig,
        height: Milimeters,
    ) -> Self::Output;
}

/// Format agnostic sliced file info.
pub struct SliceInfo {
    pub layers: u32,
    pub resolution: Vector2<u32>,
    pub size: Vector3<Milimeters>,
    pub bottom_layers: u32,
}

#[derive(Clone)]
pub struct Layer {
    pub data: Arc<Vec<Run>>,
    pub area: u64,
    pub height: Milimeters,

    /// If this exposure is not derived directly from the slice config.
    pub unique_exposure: bool,
    pub exposure: ExposureConfig,
}

impl Layer {
    pub fn new(data: Vec<Run>, height: Milimeters, exposure: ExposureConfig) -> Self {
        Self::new_arc(Arc::new(data), height, exposure)
    }

    pub fn new_arc(data: Arc<Vec<Run>>, height: Milimeters, exposure: ExposureConfig) -> Self {
        let area = data
            .iter()
            .filter(|x| x.value > 0)
            .fold(0, |acc, run| acc + run.length);

        Self {
            data,
            area,
            height,

            unique_exposure: false,
            exposure,
        }
    }
}

pub fn print_time<'a, I: Iterator<Item = &'a Layer>>(layers: I) -> Seconds {
    layers
        .map(|x| x.exposure.print_time())
        .fold(Seconds::new(0.0), |a, b| a + b)
}

pub type Polygon = Vec<Vector2<f32>>;
pub type VectorLayer = Vec<Polygon>;
