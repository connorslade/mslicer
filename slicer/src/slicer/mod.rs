use common::{progress::Progress, slice::SliceConfig};

use crate::mesh::Mesh;

mod slice_raster;
pub mod slice_vector;

const SEGMENT_LAYERS: usize = 100;

/// Used to slice a mesh.
pub struct Slicer {
    pub slice_config: SliceConfig,
    models: Vec<SlicerModel>,

    layers: u32,
    progress: Progress,
}

pub struct SlicerModel {
    pub mesh: Mesh,
    pub exposure: u8,
}

impl Slicer {
    /// Creates a new slicer given a slice config, list of models, and their relative exposures.
    pub fn new(slice_config: SliceConfig, models: Vec<SlicerModel>) -> Self {
        let max_z = models.iter().fold(0_f32, |max, model| {
            let verts = model.mesh.vertices().iter();
            let z = verts.fold(0_f32, |max, &f| max.max(model.mesh.transform(&f).z));
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

    pub fn layer_count(&self) -> u32 {
        self.layers
    }

    /// Gets an instance of the slicing [`Progress`] struct.
    pub fn progress(&self) -> Progress {
        self.progress.clone()
    }
}
