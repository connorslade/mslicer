use common::image::Image;
use egui::{Context, Ui};
use image::GrayImage;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{app::App, ui::components::dragger};
use goo_format::{File as GooFile, LayerDecoder, LayerEncoder};

use super::Plugin;

pub struct AntiAliasPlugin {
    enabled: bool,
    radius: f32,
}

impl Plugin for AntiAliasPlugin {
    fn name(&self) -> &'static str {
        "Anti Aliasing"
    }

    fn ui(&mut self, _app: &mut App, ui: &mut Ui, _ctx: &Context) {
        ui.label("Applies a blur to each layer to smooth the edges.");
        ui.checkbox(&mut self.enabled, "Enabled");

        ui.add_space(8.0);
        dragger(ui, "Radius", &mut self.radius, |x| {
            x.speed(0.1).clamp_range(0.1..=10.0)
        });
    }

    fn post_slice(&self, _app: &App, goo: &mut GooFile) {
        if !self.enabled {
            return;
        }

        let (width, height) = (
            goo.header.x_resolution as usize,
            goo.header.y_resolution as usize,
        );

        goo.layers.par_iter_mut().for_each(|layer| {
            let decoder = LayerDecoder::new(&layer.data);
            let raw_image = Image::from_decoder(width, height, decoder).take();

            let image = GrayImage::from_raw(width as u32, height as u32, raw_image).unwrap();
            let image = imageproc::filter::gaussian_blur_f32(&image, self.radius);

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
    Box::new(AntiAliasPlugin {
        enabled: false,
        radius: 1.0,
    })
}
