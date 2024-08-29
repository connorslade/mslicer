use common::image::Image;
use egui::{Context, Ui};
use tracing::info;

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

        ui.add_space(8.0);
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

        for layer in goo
            .layers
            .iter_mut()
            .take(goo.header.bottom_layers as usize)
        {
            let decoder = LayerDecoder::new(&layer.data);
            let image = Image::from_decoder(width, height, decoder);
            let image = erode(image, self.intensity_multiplier / 100.0, x_radius, y_radius);

            let mut new_layer = LayerEncoder::new();
            for run in image.runs() {
                new_layer.add_run(run.length, run.value)
            }

            let (data, checksum) = new_layer.finish();
            layer.data = data;
            layer.checksum = checksum;
        }
    }
}

pub fn get_plugin() -> Box<dyn Plugin> {
    Box::new(ElephantFootFixerPlugin {
        enabled: false,
        inset_distance: 2.0,
        intensity_multiplier: 30.0,
    })
}

pub fn erode(image: Image, intensity: f32, x_radius: usize, y_radius: usize) -> Image {
    let mut new_layer = image.clone();

    for x in 0..image.size.x {
        'outer: for y in 0..image.size.y {
            let pixel = image.get_pixel(x, y);
            if pixel == 0 {
                continue;
            }

            // if there are any black pixels in the radius, multiply the pixel by the intensity multiplier
            for xp in x.saturating_sub(x_radius)..(x + x_radius).min(image.size.x) {
                for yp in y.saturating_sub(y_radius)..(y + y_radius).min(image.size.y) {
                    if image.get_pixel(xp, yp) == 0 {
                        new_layer.set_pixel(x, y, (pixel as f32 * intensity) as u8);
                        continue 'outer;
                    }
                }
            }
        }
    }

    new_layer
}
