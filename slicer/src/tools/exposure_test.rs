use common::{container::Image, progress::Progress, slice::SliceConfig};
use nalgebra::{Vector2, Vector3};

#[derive(Clone)]
pub struct ExposureTestGenerator {
    pub size: Vector3<u32>,
    pub steps: u32,
}

impl ExposureTestGenerator {
    pub fn generate(
        &self,
        config: &SliceConfig,
        progress: &Progress,
    ) -> impl Iterator<Item = Image> {
        progress.set_total(self.size.z as u64);
        (0..self.size.z).map(move |_layer| {
            let mut image = Image::blank(config.platform_resolution.cast());
            image.rect((Vector2::zeros(), self.size.xy().cast()), 255);
            progress.add_complete(1);
            image
        })
    }
}

impl Default for ExposureTestGenerator {
    fn default() -> Self {
        Self {
            size: Vector3::new(10, 10, 10),
            steps: 32,
        }
    }
}
