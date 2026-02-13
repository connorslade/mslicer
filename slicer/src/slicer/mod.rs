use common::{
    progress::Progress,
    slice::{DynSlicedFile, Format, SliceConfig},
};

use crate::{mesh::Mesh, slicer::slice_vector::SvgFile};

mod slice_raster;
mod slice_vector;

const SEGMENT_LAYERS: usize = 100;

/// Used to slice a mesh.
pub struct Slicer {
    slice_config: SliceConfig,
    models: Vec<Mesh>,
    layers: u32,
    progress: Progress,
}

impl Slicer {
    /// Creates a new slicer given a slice config and list of models.
    pub fn new(slice_config: SliceConfig, models: Vec<Mesh>) -> Self {
        let max_z = models.iter().fold(0_f32, |max, model| {
            let verts = model.vertices().iter();
            let z = verts.fold(0_f32, |max, &f| max.max(model.transform(&f).z));
            max.max(z)
        });

        let slice = slice_config.slice_height;
        let max_layers = (slice_config.platform_size.z / slice).ceil() as u32;
        let layers = ((max_z / slice).raw().ceil() as u32).min(max_layers);

        let progress = Progress::new();
        progress.set_total(layers as u64);

        Self {
            slice_config,
            models,
            layers,
            progress,
        }
    }

    pub fn slice_config(&self) -> &SliceConfig {
        &self.slice_config
    }

    pub fn layer_count(&self) -> u32 {
        self.layers
    }

    /// Gets an instance of the slicing [`Progress`] struct.
    pub fn progress(&self) -> Progress {
        self.progress.clone()
    }

    pub fn slice(&self) -> (DynSlicedFile, u64) {
        match self.slice_config.format {
            Format::Goo => {
                let result = self.slice_raster::<goo_format::LayerEncoder>();
                let voxels = result.voxels;
                let file = Box::new(goo_format::File::from_slice_result(result));
                (file, voxels)
            }
            Format::Ctb => {
                let result = self.slice_raster::<ctb_format::LayerEncoder>();
                let voxels = result.voxels;
                let file = Box::new(ctb_format::File::from_slice_result(result));
                (file, voxels)
            }
            Format::NanoDLP => {
                let result = self.slice_raster::<nanodlp_format::LayerEncoder>();
                let voxels = result.voxels;
                let file = Box::new(nanodlp_format::File::from_slice_result(result));
                (file, voxels)
            }
            Format::Svg => (Box::new(SvgFile::new(self.slice_vector())), 0),
        }
    }
}
