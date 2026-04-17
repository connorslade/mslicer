use std::iter::{self, repeat_n};

use common::{
    container::Image,
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use nalgebra::{Vector2, Vector3};

#[derive(Clone)]
pub struct ExposureTest {
    pub size: Vector3<f32>,
    pub supports: Supports,
    pub steps: u32,
}

#[derive(Clone)]
pub struct Supports {
    pub enabled: bool,
    pub height: f32,
    pub spacing: f32,
}

impl ExposureTest {
    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let slice_height = config.slice_height.get::<Milimeter>();
        let layers = (self.size.z / slice_height).round() as u64;
        let support_layers = if self.supports.enabled {
            (self.supports.height / slice_height).round() as u64
        } else {
            0
        };
        progress.set_total(support_layers + self.steps as u64);

        let size = config.mm_to_px(self.size.xy()).map(|x| x.round() as u32);
        let min = ((config.platform_resolution - size.xy()) / 2).cast();
        let max = min + size.xy().cast();

        let mut top = Image::blank(config.platform_resolution.cast());
        let mut body = Image::blank(config.platform_resolution.cast());
        body.rect((min, max), 255);
        let raft = body.clone();

        let strip_width = (size.x / self.steps) as usize;
        for (i, value) in self.steps() {
            let a = Vector2::new(min.x + strip_width * i, min.y);
            let b = Vector2::new(a.x + strip_width, max.y);
            top.rect((a, b), value);

            let a = Vector2::new(min.x + strip_width * i, max.y - 1);
            let b = Vector2::new(a.x + strip_width, a.y + 1);
            body.rect((a, b), value);
            progress.add_complete(1);
        }

        let [raft, top, body] = [raft, top, body].map(|x| x.runs().collect::<Vec<_>>());
        (0..support_layers)
            .map(move |layer| {
                if layer < config.first_layers as u64 {
                    return raft.clone();
                }

                let mut image = Image::blank(config.platform_resolution.cast());
                let t = layer as f32 / (support_layers - 1) as f32;
                let r = if t >= 0.5 {
                    lerp(0.5, 0.3, t / 0.5 - 1.0)
                } else {
                    0.5
                };

                let step = config
                    .mm_to_px(Vector2::repeat(self.supports.spacing))
                    .map(|x| x.round() as usize);
                let r = (config.mm_to_px(Vector2::repeat(r))).map(|x| x.round() as u32);

                let n = (max - min).component_div(&step);
                let offset = ((max - min) - n.component_mul(&step)) / 2;
                for y in 0..=n.y {
                    for x in 0..=n.x {
                        let pos = min + offset + Vector2::new(x, y).component_mul(&step);
                        image.circle(pos, r, 255);
                    }
                }

                image.runs().collect::<Vec<_>>()
            })
            .inspect(|_| progress.add_complete(1))
            .chain(repeat_n(body, layers as usize))
            .chain(iter::once(top))
            .enumerate()
            .map(|(layer, data)| Layer {
                data,
                exposure: config.exposure_config(layer as u32).clone(),
            })
            .collect()
    }

    fn steps(&self) -> impl Iterator<Item = (usize, u8)> {
        (0..self.steps as usize).map(|i| {
            let t = i as f32 / (self.steps - 1) as f32;
            let value = (t * 255.0) as u8;
            (i, value)
        })
    }
}

impl Default for ExposureTest {
    fn default() -> Self {
        Self {
            size: Vector3::new(150.0, 10.0, 5.0),
            supports: Supports {
                enabled: false,
                height: 3.0,
                spacing: 2.0,
            },
            steps: 15,
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    b * t + a * (1.0 - t)
}
