use image::RgbaImage;
use nalgebra::{Vector2, Vector3};

mod config;
mod format;
mod layer_iter;
pub use config::{ExposureConfig, SliceConfig};
pub use format::Format;
pub use layer_iter::SliceLayerIterator;

use crate::{
    container::{Image, Run, rle},
    progress::Progress,
    serde::DynamicSerializer,
    units::Milimeters,
};

pub type DynSlicedFile = Box<dyn SlicedFile + Send + Sync>;

pub trait SlicedFile {
    fn serialize(&self, ser: &mut DynamicSerializer, progress: Progress);
    fn set_preview(&mut self, preview: &RgbaImage);
    fn info(&self) -> SliceInfo;
    fn format(&self) -> Format;

    fn runs(&self, layer: usize) -> Box<dyn Iterator<Item = Run> + '_>;
    fn overwrite_layer(&mut self, layer: usize, image: Image);
    fn decode_layer(&self, layer: usize, image: &mut [u8]) {
        let decoder = self.runs(layer);
        rle::decode_into(decoder, image);
    }
    fn read_layer(&self, layer: usize) -> Image {
        Image::from_decoder(self.info().resolution.cast(), self.runs(layer))
    }
}

pub trait EncodableLayer {
    type Output: Send;

    fn new(platform: Vector2<u32>) -> Self;
    fn add_run(&mut self, length: u64, value: u8);
    fn finish(self, layer: u32, config: &SliceConfig) -> Self::Output;
}

pub struct SliceInfo {
    pub layers: u32,
    pub resolution: Vector2<u32>,
    pub size: Vector3<Milimeters>,
    pub bottom_layers: u32,
}

pub struct SliceResult<'a, Layer> {
    pub layers: Vec<Layer>,
    pub voxels: u64,
    pub slice_config: &'a SliceConfig,
}

pub type Polygon = Vec<Vector2<f32>>;
pub type VectorLayer = Vec<Polygon>;
pub struct VectorSliceResult<'a> {
    pub layers: Vec<VectorLayer>,
    pub slice_config: &'a SliceConfig,
}
