use std::iter::repeat_n;

use common::{
    container::{Image, Run},
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use nalgebra::Vector2;

#[derive(Clone)]
pub struct InternalExposureTest {
    pub size: Vector2<f32>,   // length, width
    pub wall: f32,            // wall size (mm)
    pub border_exposure: f32, // fraction [0,1]
}

impl InternalExposureTest {
    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let slice_height = config.slice_height.get::<Milimeter>();
        let layers = (self.size.y / slice_height).round() as usize;
        progress.set_total(layers as u64);

        let outer = self.layer(config, false);
        let inner = self.layer(config, true);

        let wall_layers = (self.wall / slice_height).round() as usize;
        repeat_n(outer.clone(), wall_layers)
            .chain(repeat_n(inner, layers - wall_layers * 2))
            .chain(repeat_n(outer, wall_layers))
            .enumerate()
            .map(|(layer, data)| Layer {
                data,
                exposure: config.exposure_config(layer as u32).clone(),
            })
            .inspect(|_| progress.add_complete(1))
            .collect()
    }

    fn layer(&self, config: &SliceConfig, internal: bool) -> Vec<Run> {
        let size = config.mm_to_px(self.size).map(|x| x.round() as u32);
        let min = ((config.platform_resolution - size.xy()) / 2).cast();
        let max = min + size.xy().cast();

        let mut image = Image::blank(config.platform_resolution.cast());
        image.rect((min, max), (self.border_exposure * 255.0) as u8);

        if internal {
            let diff = config
                .mm_to_px(Vector2::repeat(self.wall))
                .map(|x| x.round() as usize);
            let min_diff = Vector2::new(min.x + diff.x, min.y + diff.y);
            let max_diff = Vector2::new(max.x - diff.x, max.y - diff.y);
            let segment_width = (max_diff.x - min_diff.x) as f32 / 256.0;

            for i in 0..=255 {
                let p_min = min_diff + Vector2::x() * (i as f32 * segment_width).round() as usize;
                let p_max = Vector2::new(min_diff.x, max_diff.y)
                    + Vector2::x() * ((i + 1) as f32 * segment_width).round() as usize;
                image.rect((p_min, p_max), i as u8);
            }
        }

        image.runs().collect()
    }
}

impl Default for InternalExposureTest {
    fn default() -> Self {
        Self {
            size: Vector2::new(100.0, 20.0),
            wall: 2.0,
            border_exposure: 1.0,
        }
    }
}
