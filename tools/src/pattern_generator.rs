use common::{
    container::Run,
    progress::Progress,
    slice::{Layer, SliceConfig},
};

#[derive(Clone)]
pub struct PatternGenerator {}

impl PatternGenerator {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let pixels = config.platform_resolution.x * config.platform_resolution.y;
        progress.set_total(1);

        let mut data = Vec::new();
        for i in 0..pixels {
            data.push(Run::new(1, (i % 255) as u8));
        }

        progress.set_finished();
        vec![Layer::new(
            data,
            config.default_height(0),
            config.exposure_config(0).into_owned(),
        )]
    }
}

impl Default for PatternGenerator {
    fn default() -> Self {
        Self {}
    }
}
