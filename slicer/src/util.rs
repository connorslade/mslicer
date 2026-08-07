use std::{borrow::Borrow, sync::Arc};

use common::{
    container::rle::downsample::RunFlattenExt,
    slice::{
        self, DynSlicedFile, EncodableLayer, SliceConfig, VectorLayer,
        format::{RasterFormat, VectorFormat},
    },
};

use crate::slicer::vector::SvgFile;

pub fn export_raster<Layers, Layer>(
    config: &SliceConfig,
    layers: Layers,
    voxels: u64,
    format: RasterFormat,
) -> DynSlicedFile
where
    Layers: IntoIterator<Item = Layer>,
    Layer: Borrow<slice::Layer>,
{
    match format {
        RasterFormat::Goo => Box::new(goo_format::File::from_layers(
            config,
            encode_raster_layers::<goo_format::LayerEncoder, _, _>(config, layers),
        )),
        RasterFormat::Ctb => Box::new(ctb_format::File::from_layers(
            config,
            encode_raster_layers::<ctb_format::LayerEncoder, _, _>(config, layers),
        )),
        RasterFormat::NanoDLP => Box::new(nanodlp_format::File::from_layers(
            config,
            encode_raster_layers::<nanodlp_format::LayerEncoder, _, _>(config, layers),
            voxels,
        )),
    }
}

pub fn export_vector(
    config: &SliceConfig,
    layers: Arc<Vec<VectorLayer>>,
    format: VectorFormat,
) -> DynSlicedFile {
    match format {
        VectorFormat::Svg => Box::new(SvgFile::new(config.platform_resolution.xy(), layers)),
    }
}

pub fn encode_raster_layers<Encoder, Layers, Layer>(
    config: &SliceConfig,
    layers: Layers,
) -> Vec<Encoder::Output>
where
    Encoder: EncodableLayer,
    Layers: IntoIterator<Item = Layer>,
    Layer: Borrow<slice::Layer>,
{
    layers
        .into_iter()
        .map(|layer| {
            let layer = layer.borrow();
            let mut encoder = Encoder::new(config.platform_resolution);

            // The runs need to be 'flattened' (adjacent runs with the same
            // value combined) because due to the way anti-aliasing is
            // implemented no runs (excluding the first and last run) will
            // continue for multiple scan lines.
            //
            // This mainly affects the fully black (value = 0) runs.
            (layer.data.iter().copied())
                .run_flatten()
                .for_each(|run| encoder.add_run(run.length, run.value));
            encoder.finish(config, &layer.exposure, layer.height)
        })
        .collect()
}
