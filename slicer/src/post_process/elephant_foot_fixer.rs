use std::{mem, time::Instant};

use image::Luma;
use imageproc::{morphology::Mask, point::Point};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::format::FormatSliceFile;

#[derive(Serialize, Deserialize)]
pub struct ElephantFootFixer {
    pub enabled: bool,
    pub inset_distance: f32,
    pub intensity_multiplier: f32,
}

impl ElephantFootFixer {
    pub fn post_slice(&self, file: &mut FormatSliceFile) {
        if !self.enabled {
            return;
        }

        let info = file.info();
        let (width, height) = (info.resolution.x as usize, info.resolution.y as usize);

        let (x_radius, y_radius) = (
            (self.inset_distance * (width as f32 / info.size.x)) as usize,
            (self.inset_distance * (height as f32 / info.size.y)) as usize,
        );
        info!(
            "Eroding {} bottom layers with radius ({}, {})",
            info.bottom_layers, x_radius, y_radius
        );

        let intensity = self.intensity_multiplier / 100.0;
        let mask = generate_mask(x_radius, y_radius);

        let darken = |value: u8| (value as f32 * intensity).round() as u8;

        let start = Instant::now();
        file.iter_mut_layers()
            .take(info.bottom_layers as usize)
            .par_bridge()
            .for_each(|mut layer| {
                let erode = imageproc::morphology::grayscale_erode(&layer, &mask);
                for (x, y, pixel) in layer.enumerate_pixels_mut() {
                    if erode.get_pixel(x, y)[0] == 0 && pixel[0] != 0 {
                        *pixel = Luma([darken(pixel[0])]);
                    }
                }
            });

        info!("Eroded bottom layers in {:?}", start.elapsed());
    }
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
