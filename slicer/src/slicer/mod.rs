use common::{config::SliceConfig, format::Format, progress::Progress};

use crate::{format::FormatSliceResult, mesh::Mesh};

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
        let max_layers = (slice_config.platform_size.z / slice).raw().ceil() as u32;
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

    pub fn slice_format(&self) -> FormatSliceResult<'_> {
        type GooEncoder = goo_format::LayerEncoder;
        type CtbEncoder = ctb_format::LayerEncoder;
        type NanoDLPEncoder = nanodlp_format::LayerEncoder;

        match self.slice_config.format {
            Format::Goo => FormatSliceResult::Goo(self.slice::<GooEncoder>()),
            Format::Ctb => FormatSliceResult::Ctb(self.slice::<CtbEncoder>()),
            Format::NanoDLP => FormatSliceResult::NanoDLP(self.slice::<NanoDLPEncoder>()),
            Format::Svg => FormatSliceResult::Svg(self.slice_vector()),
        }
    }
}
