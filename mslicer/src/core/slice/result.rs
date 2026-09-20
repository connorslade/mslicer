use std::{sync::Arc, time::Duration};

use common::{
    misc::human_duration,
    progress::Progress,
    slice::{DynSlicedFile, Layer, SliceConfig, VectorLayer, format::Format},
    units::{Miliseconds, Milliliters, Seconds},
};
use image::RgbaImage;
use rayon::iter::IntoParallelRefIterator;
use slicer::{slicer::vector::SvgFile, util};

use crate::core::slice::annotations::Annotations;

pub struct SliceResult {
    pub config: SliceConfig,
    pub elapsed: Duration,
    pub fresh: bool,
    pub sliced: bool,

    pub variable_layer_height: bool,
    pub inner: GenericSliceResult,
}

#[derive(Clone)]
pub enum GenericSliceData {
    Raster { data: Vec<Layer>, voxels: u64 },
    Vector { data: Arc<Vec<VectorLayer>> },
}

pub enum GenericSliceResult {
    Raster(RasterSliceResult),
    Vector(VectorSliceResult),
}

pub struct RasterSliceResult {
    pub layers: Vec<Layer>,
    pub annotations: Arc<Annotations>,
    pub detected_islands: bool,

    pub voxels: u64,
    pub volume: Milliliters,
    pub print_time: Seconds,
}

pub struct VectorSliceResult {
    pub layers: Arc<Vec<VectorLayer>>,
}

impl SliceResult {
    pub fn completion(&self) -> String {
        let time = self.elapsed.as_millis() as f32;
        human_duration(Miliseconds::new(time))
    }

    pub fn layers(&self) -> usize {
        match self.slice_data() {
            GenericSliceData::Raster { data, .. } => data.len(),
            GenericSliceData::Vector { data } => data.len(),
        }
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

    pub fn as_raster_mut(&mut self) -> Option<&mut RasterSliceResult> {
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
        progress: &Progress,
        config: &SliceConfig,
        preview_image: &RgbaImage,
        format: Format,
    ) -> DynSlicedFile {
        match &self {
            GenericSliceData::Raster { data, voxels } => {
                progress.set_total(data.len() as u64);
                let format = format.as_raster().unwrap();
                let mut file =
                    util::export_raster(progress, config, data.par_iter(), *voxels, format);
                file.set_preview(preview_image);
                progress.set_finished();
                file
            }
            GenericSliceData::Vector { data } => {
                progress.set_total(1);
                progress.set_finished();

                let platform = config.platform_resolution.xy();
                let file = SvgFile::new(platform, data.clone());
                Box::new(file)
            }
        }
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
