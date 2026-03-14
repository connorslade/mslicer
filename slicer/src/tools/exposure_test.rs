use std::iter;

use common::{container::Image, progress::Progress, slice::SliceConfig, units::Milimeter};
use nalgebra::{Vector2, Vector3};

#[derive(Clone)]
pub struct ExposureTest {
    pub size: Vector3<f32>,
    pub steps: u32,
}

impl ExposureTest {
    pub fn generate(
        &self,
        config: &SliceConfig,
        progress: &Progress,
    ) -> impl Iterator<Item = Image> {
        let layers = (self.size.z / config.slice_height.get::<Milimeter>()).round() as u64;
        let size = (self.size.xy())
            .component_mul(&config.platform_resolution.cast())
            .component_div(&config.platform_size.xy().map(|x| x.get::<Milimeter>()))
            .map(|x| x.round() as u32);

        progress.set_total(layers);
        let min = ((config.platform_resolution - size.xy()) / 2).cast();
        let max = min + size.xy().cast();

        let mut top = Image::blank(config.platform_resolution.cast());
        let strip_width = (size.x / self.steps) as usize;
        for i in 0..self.steps as usize {
            let t = i as f32 / (self.steps - 1) as f32;
            let value = (t * 255.0) as u8;

            top.rect(
                (
                    Vector2::new(min.x + strip_width * i, min.y),
                    Vector2::new(min.x + strip_width * (i + 1), max.y),
                ),
                value,
            );
        }
        progress.add_complete(1);

        (0..layers)
            .map(move |_layer| {
                let mut image = Image::blank(config.platform_resolution.cast());
                image.rect((min, max), 255);
                progress.add_complete(1);
                image
            })
            .chain(iter::once(top))
    }
}

impl Default for ExposureTest {
    fn default() -> Self {
        Self {
            size: Vector3::new(150.0, 10.0, 5.0),
            steps: 15,
        }
    }
}
