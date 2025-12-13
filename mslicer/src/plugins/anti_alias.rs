use egui::{Context, Ui};
use libblur::{
    AnisotropicRadius, BlurImageMut, EdgeMode, EdgeMode2D, FastBlurChannels, ThreadingPolicy,
};
use rayon::iter::{ParallelBridge, ParallelIterator};
use slicer::format::FormatSliceFile;

use crate::{app::App, ui::components::dragger};

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
            x.speed(0.1).range(0.1..=10.0)
        });
    }

    fn post_slice(&self, _app: &App, file: &mut FormatSliceFile) {
        if !self.enabled {
            return;
        }

        file.iter_mut_layers().par_bridge().for_each(|mut layer| {
            let (width, height) = (layer.width(), layer.height());
            let mut image =
                BlurImageMut::borrow(&mut layer, width, height, FastBlurChannels::Plane);
            libblur::fast_gaussian_next(
                &mut image,
                AnisotropicRadius::new(self.radius as u32),
                ThreadingPolicy::Adaptive,
                EdgeMode2D::new(EdgeMode::Clamp),
            )
            .unwrap();
        });
    }
}

pub fn get_plugin() -> Box<dyn Plugin> {
    Box::new(AntiAliasPlugin {
        enabled: false,
        radius: 1.0,
    })
}
