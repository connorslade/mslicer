use std::borrow::Borrow;

use common::slice::{self, DynSlicedFile, EncodableLayer, Format, SliceConfig};

pub fn export<Layers, Layer>(config: &SliceConfig, layers: Layers, voxels: u64) -> DynSlicedFile
where
    Layers: IntoIterator<Item = Layer>,
    Layer: Borrow<slice::Layer>,
{
    match config.format {
        Format::Goo => Box::new(goo_format::File::from_layers(
            config,
            encode_layers::<goo_format::LayerEncoder, _, _>(config, layers),
        )),
        Format::Ctb => Box::new(ctb_format::File::from_layers(
            config,
            encode_layers::<ctb_format::LayerEncoder, _, _>(config, layers),
        )),
        Format::NanoDLP => Box::new(nanodlp_format::File::from_layers(
            config,
            encode_layers::<nanodlp_format::LayerEncoder, _, _>(config, layers),
            voxels,
        )),
        Format::Svg => panic!(),
    }
}

pub fn encode_layers<Encoder, Layers, Layer>(
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
        .enumerate()
        .map(|(i, layer)| {
            let layer = layer.borrow();
            let mut encoder = Encoder::new(config.platform_resolution);
            (layer.data.iter()).for_each(|run| encoder.add_run(run.length, run.value));
            encoder.finish(i as u32, config, &layer.exposure)
        })
        .collect()
}

// pub fn encode_layers<Layer: EncodableLayer>(
//     slice_config: &SliceConfig,
//     layers: impl Iterator<Item = Image>,
// ) -> (Vec<Layer::Output>, u64) {
//     let mut voxels = 0;
//     let layers = layers.into_iter().enumerate().map(|(i, image)| {
//         let mut encoder = Layer::new(slice_config.platform_resolution);
//         for run in image.runs() {
//             encoder.add_run(run.length, run.value);
//             (run.value > 0).then(|| voxels += run.length);
//         }

//         encoder.finish(i as u32, slice_config)
//     });

//     (layers.collect(), voxels)
// }
