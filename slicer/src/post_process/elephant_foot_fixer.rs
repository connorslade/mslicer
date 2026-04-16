use std::{mem, time::Instant};

use common::{
    container::{Image, rle},
    progress::Progress,
    serde::{Deserializer, Serializer},
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use image::{GrayImage, Luma};
use imageproc::{morphology::Mask, point::Point};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone, Serialize, Deserialize)]
pub struct ElephantFootFixer {
    pub enabled: bool,
    pub inset_distance: f32,
    pub intensity_multiplier: f32,
}

impl ElephantFootFixer {
    pub fn post_slice(&self, config: &SliceConfig, layers: &mut [Layer], progress: Progress) {
        if !self.enabled {
            return;
        }

        let [width, height] = *config.platform_resolution.as_ref();
        let platform = config.platform_size;

        let (x_radius, y_radius) = (
            (self.inset_distance * (width as f32 / platform.x).get::<Milimeter>()) as usize,
            (self.inset_distance * (height as f32 / platform.y).get::<Milimeter>()) as usize,
        );
        info!(
            "Eroding {} bottom layers with radius ({}, {})",
            config.first_layers, x_radius, y_radius
        );

        let intensity = self.intensity_multiplier / 100.0;
        let mask = generate_mask(x_radius, y_radius);

        let darken = |value: u8| (value as f32 * intensity).round() as u8;

        let start = Instant::now();
        progress.set_total(config.first_layers as u64);

        layers
            .iter_mut()
            .take(config.first_layers as usize)
            .par_bridge()
            .for_each(|layer| {
                let inner = rle::decode_vec(&layer.data);
                let mut image = GrayImage::from_raw(width, height, inner).unwrap();

                let erode = imageproc::morphology::grayscale_erode(&image, &mask);
                for (x, y, pixel) in image.enumerate_pixels_mut() {
                    if erode.get_pixel(x, y)[0] == 0 && pixel[0] != 0 {
                        *pixel = Luma([darken(pixel[0])]);
                    }
                }

                let image = Image::from_raw(config.platform_resolution.cast(), image.into_raw());
                layer.data = image.runs().collect();
            });

        progress.set_finished();
        info!("Eroded bottom layers in {:?}", start.elapsed());
    }
}

fn generate_mask(width: usize, height: usize) -> Mask {
    let (width, height) = ((width / 2) as i16, (height / 2) as i16);

    let points = (-width..=width)
        .cartesian_product(-height..=height)
        .map(|(x, y)| Point::new(x, y))
        .collect::<Vec<_>>();

    new_mask_unsafe(points)
}

fn new_mask_unsafe(points: Vec<Point<i16>>) -> Mask {
    unsafe { mem::transmute(points) }
}

impl Default for ElephantFootFixer {
    fn default() -> Self {
        Self {
            enabled: false,
            inset_distance: 0.5,
            intensity_multiplier: 30.0,
        }
    }
}

impl ElephantFootFixer {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_bool(self.enabled);
        ser.write_f32_be(self.inset_distance);
        ser.write_f32_be(self.intensity_multiplier);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            enabled: des.read_bool(),
            inset_distance: des.read_f32_be(),
            intensity_multiplier: des.read_f32_be(),
        }
    }
}
