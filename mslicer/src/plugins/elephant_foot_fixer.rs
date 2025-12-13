use std::{mem, time::Instant};

use egui::{Align, Context, Layout, Ui};
use egui_phosphor::regular::{INFO, WARNING};
use image::Luma;
use imageproc::{morphology::Mask, point::Point};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use slicer::format::FormatSliceFile;
use tracing::info;

use crate::{
    app::App,
    ui::components::{dragger, dragger_tip},
};

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
        ui.horizontal(|ui| {
            dragger(ui, "Inset Distance", &mut self.inset_distance, |x| {
                x.speed(0.1).suffix("mm")
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.label(INFO).on_hover_text(
                    "The distance in from the edges that will have a reduced intensity.",
                );
                ui.label(WARNING)
                    .on_hover_text("Larger values will drastically increase post-processing time.");
                ui.add_space(ui.available_width());
            })
        });

        dragger_tip(
            ui,
            "Intensity",
            "This percent will be multiplied by the pixel values of the edge pixels.",
            &mut self.intensity_multiplier,
            |x| x.range(0.0..=100.0).speed(1).suffix("%"),
        );
    }

    fn post_slice(&self, _app: &App, file: &mut FormatSliceFile) {
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

pub fn get_plugin() -> Box<dyn Plugin> {
    Box::new(ElephantFootFixerPlugin {
        enabled: false,
        inset_distance: 0.5,
        intensity_multiplier: 30.0,
    })
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
