use nalgebra::Vector2;

mod config;
mod format;
pub use config::{ExposureConfig, SliceConfig};
pub use format::Format;

pub struct SliceResult<'a, Layer> {
    pub layers: Vec<Layer>,
    pub voxels: u64,
    pub slice_config: &'a SliceConfig,
}

pub struct VectorSliceResult<'a> {
    pub layers: Vec<VectorLayer>,
    pub slice_config: &'a SliceConfig,
}

pub struct VectorLayer {
    pub polygons: Vec<Vec<Vector2<f32>>>,
}

pub trait EncodableLayer {
    type Output: Send;

    fn new(platform: Vector2<u32>) -> Self;
    fn add_run(&mut self, length: u64, value: u8);
    fn finish(self, layer: u32, config: &SliceConfig) -> Self::Output;
}
