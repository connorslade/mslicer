use std::{borrow::Borrow, sync::Arc};

use anyhow::Result;

use common::{
    container::rle::downsample::RunFlattenExt,
    progress::Progress,
    serde::SliceDeserializer,
    slice::{
        self, DynSlicedFile, EncodableLayer, Layer, SliceConfig, VectorLayer,
        format::{RasterFormat, VectorFormat},
    },
};
use image::RgbaImage;

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

// todo: make some kinda generic decoder maybe
pub fn load_sliced(
    progress: &Progress,
    format: &RasterFormat,
    data: &[u8],
) -> Result<(SliceConfig, Vec<Layer>, RgbaImage)> {
    let mut des = SliceDeserializer::new(data);
    match format {
        RasterFormat::Goo => {
            let file = goo_format::File::deserialize(&mut des)?;
            progress.set_total(file.layers.len() as _);

            let config = file.header.into_slice_config();
            let layers = (file.layers.iter())
                .map(|x| {
                    progress.add_complete(1);
                    x.into_layer()
                })
                .collect();
            let image = file.header.big_preview.to_image();
            progress.set_finished();
            Ok((config, layers, image))
        }
        RasterFormat::Ctb => todo!(),
        RasterFormat::NanoDLP => todo!(),
    }
}
