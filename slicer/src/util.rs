// todo: move to common and ex-export through msla_format

use std::{
    borrow::Borrow,
    io::{Read, Seek},
    sync::Arc,
};

use anyhow::{Ok, Result, bail};
use common::{
    container::rle::downsample::RunFlattenExt,
    progress::Progress,
    serde::SliceDeserializer,
    slice::{
        self, DynSlicedFile, EncodableLayer, SliceConfig, SlicedFile, VectorLayer,
        format::{CtbFormat, RasterFormat, VectorFormat},
    },
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use {
    ctb_format::shared as ctb, ctb_format::v4 as ctb4, ctb_format::v5 as ctb5, goo_format as goo,
    nanodlp_format as nanodlp,
};

use crate::slicer::vector::SvgFile;

// todo: make a new format crate that has all this

pub fn export_raster<Layers, Layer>(
    progress: &Progress,
    config: &SliceConfig,
    layers: Layers,
    voxels: u64,
    format: RasterFormat,
) -> DynSlicedFile
where
    Layers: IntoParallelIterator<Item = Layer>,
    Layer: Borrow<slice::Layer>,
{
    match format {
        RasterFormat::Goo => Box::new(goo::File::from_layers(
            config,
            encode_raster_layers::<goo::LayerEncoder, _, _>(progress, config, layers),
        )),
        RasterFormat::Ctb(ctb) => match ctb {
            CtbFormat::Encrypted => Box::new(ctb5::File::from_layers(
                config,
                encode_raster_layers::<ctb::LayerEncoderV5, _, _>(progress, config, layers),
            )),
            CtbFormat::Legacy => Box::new(ctb4::File::from_layers(
                config,
                encode_raster_layers::<ctb::LayerEncoderV4, _, _>(progress, config, layers),
            )),
            CtbFormat::Unknown => unreachable!(),
        },
        RasterFormat::NanoDLP => Box::new(nanodlp::File::from_layers(
            config,
            encode_raster_layers::<nanodlp::LayerEncoder, _, _>(progress, config, layers),
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
    progress: &Progress,
    config: &SliceConfig,
    layers: Layers,
) -> Vec<Encoder::Output>
where
    Encoder: EncodableLayer,
    Layers: IntoParallelIterator<Item = Layer>,
    Layer: Borrow<slice::Layer>,
{
    layers
        .into_par_iter()
        .map(|layer| {
            let layer = layer.borrow();
            let mut encoder = Encoder::new(config.platform_resolution);

            // Zero length runs can mess up the nanodlp png encoder.
            debug_assert!(layer.data.iter().all(|x| x.length > 0));

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
        .inspect(|_| progress.add_complete(1))
        .collect()
}

pub fn load_sliced(
    format: &RasterFormat,
    file: impl Read + Seek,
) -> Result<Box<dyn SlicedFile + Send + Sync>> {
    fn data(mut reader: impl Read + Seek) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        reader.read_to_end(&mut out)?;
        Ok(out)
    }

    Ok(match format {
        RasterFormat::Goo => {
            let data = data(file)?;
            let mut des = SliceDeserializer::new(&data);
            Box::new(goo::File::deserialize(&mut des)?)
        }
        RasterFormat::Ctb(ctb) => {
            let data = data(file)?;
            let mut des = SliceDeserializer::new(&data);

            match ctb.resolve_with_magic(&mut des) {
                CtbFormat::Encrypted => Box::new(ctb5::File::deserialize(&mut des)?),
                CtbFormat::Legacy => Box::new(ctb4::File::deserialize(&mut des)?),
                CtbFormat::Unknown => bail!("Unsupported CTB format version."),
            }
        }

        RasterFormat::NanoDLP => Box::new(nanodlp::File::deserialize(file)?),
    })
}
