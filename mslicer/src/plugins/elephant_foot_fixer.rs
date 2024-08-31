use std::{
    collections::{HashSet, VecDeque},
    time::Instant,
};

use common::{image::Image, rle_image::RleImage};
use egui::{Context, Ui};
use image::{GrayImage, Luma};
use imageproc::{distance_transform::Norm, morphology::Mask};
use rayon::iter::{ParallelBridge, ParallelIterator};
use tracing::{info, trace};

use crate::{app::App, ui::components::dragger_tip};
use goo_format::{File as GooFile, LayerDecoder, LayerEncoder};

use super::Plugin;

pub struct ElephantFootFixerPlugin {
    enabled: bool,
    inset_distance: f32,
    intensity_multiplier: f32,
}

impl Plugin for ElephantFootFixerPlugin {
    fn name(&self) -> &'static str {
        "Elephant Foot Fixer"
    }

    fn ui(&mut self, _app: &mut App, ui: &mut Ui, _ctx: &Context) {
        ui.label("Fixes the 'Elephant Foot' effect by exposing the edges of the bottom layers at a lower intensity. You may have to make a few test prints to find the right settings for your printer and resin.");
        ui.checkbox(&mut self.enabled, "Enabled");

        ui.add_space(8.0);
        dragger_tip(
            ui,
            "Inset Distance",
            "The distance in from the edges that will have a reduced intensity.",
            &mut self.inset_distance,
            |x| x.speed(0.1).suffix("mm"),
        );

        dragger_tip(
            ui,
            "Intensity",
            "This percent will be multiplied by the pixel values of the edge pixels.",
            &mut self.intensity_multiplier,
            |x| x.clamp_range(0.0..=100.0).speed(1).suffix("%"),
        );
    }

    fn post_slice(&self, _app: &App, goo: &mut GooFile) {
        if !self.enabled {
            return;
        }

        let (width, height) = (
            goo.header.x_resolution as usize,
            goo.header.y_resolution as usize,
        );

        let (x_radius, y_radius) = (
            (self.inset_distance * (width as f32 / goo.header.x_size)) as usize,
            (self.inset_distance * (height as f32 / goo.header.y_size)) as usize,
        );
        info!(
            "Eroding bottom layers with radius ({}, {})",
            x_radius, y_radius
        );

        let intensity = self.intensity_multiplier / 100.0;
        let darken = |value: u8| (value as f32 * intensity).round() as u8;

        goo.layers
            .iter_mut()
            .take(goo.header.bottom_layers as usize)
            .par_bridge()
            .for_each(|layer| {
                let decoder = LayerDecoder::new(&layer.data);
                let raw_image = Image::from_decoder(width, height, decoder).take();
                let mut image =
                    GrayImage::from_raw(width as u32, height as u32, raw_image).unwrap();

                let erode = imageproc::morphology::grayscale_erode(&image, &Mask::square(10));
                for (x, y, pixel) in image.enumerate_pixels_mut() {
                    if erode.get_pixel(x, y)[0] == 0 && pixel[0] != 0 {
                        *pixel = Luma([darken(pixel[0])]);
                    }
                }

                let mut new_layer = LayerEncoder::new();
                let raw_image = Image::from_raw(width, height, image.into_raw());
                for run in raw_image.runs() {
                    new_layer.add_run(run.length, run.value)
                }

                let (data, checksum) = new_layer.finish();
                layer.data = data;
                layer.checksum = checksum;
            });
    }
}

pub fn get_plugin() -> Box<dyn Plugin> {
    Box::new(ElephantFootFixerPlugin {
        enabled: false,
        inset_distance: 2.0,
        intensity_multiplier: 30.0,
    })
}
