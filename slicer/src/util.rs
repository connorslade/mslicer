use common::{
    container::Image,
    slice::{DynSlicedFile, EncodableLayer, Format, SliceConfig, SliceResult},
};

pub fn export(config: &SliceConfig, layers: impl Iterator<Item = Image>) -> (DynSlicedFile, u64) {
    match config.format {
        Format::Goo => {
            let result = encode_layers::<goo_format::LayerEncoder>(config, layers);
            let voxels = result.voxels;
            let file = Box::new(goo_format::File::from_slice_result(result));
            (file, voxels)
        }
        Format::Ctb => {
            let result = encode_layers::<ctb_format::LayerEncoder>(config, layers);
            let voxels = result.voxels;
            let file = Box::new(ctb_format::File::from_slice_result(result));
            (file, voxels)
        }
        Format::NanoDLP => {
            let result = encode_layers::<nanodlp_format::LayerEncoder>(config, layers);
            let voxels = result.voxels;
            let file = Box::new(nanodlp_format::File::from_slice_result(result));
            (file, voxels)
        }
        Format::Svg => panic!(),
    }
}

fn encode_layers<Layer: EncodableLayer>(
    slice_config: &SliceConfig,
    layers: impl Iterator<Item = Image>,
) -> SliceResult<'_, Layer::Output> {
    let mut voxels = 0;
    let layers = layers.into_iter().enumerate().map(|(i, image)| {
        let mut encoder = Layer::new(slice_config.platform_resolution);
        for run in image.runs() {
            encoder.add_run(run.length, run.value);
            (run.value > 0).then(|| voxels += run.length);
        }

        encoder.finish(i as u32, slice_config)
    });

    SliceResult {
        layers: layers.collect(),
        slice_config,
        voxels,
    }
}
