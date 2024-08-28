use egui::{Context, Ui};

use crate::{app::App, ui::components::dragger};
use goo_format::File as GooFile;

use super::Plugin;

pub struct ElephantFootFixerPlugin {
    enabled: bool,
    rest_time: f32,
    rest_layers: u32,
}

impl Plugin for ElephantFootFixerPlugin {
    fn name(&self) -> &'static str {
        "Elephant Foot Fixer"
    }

    fn ui(&mut self, _app: &mut App, ui: &mut Ui, _ctx: &Context) {
        ui.label("Fixes the 'Elephant Foot' effect by adding rest times before and after each bottom layer.");
        ui.checkbox(&mut self.enabled, "Enabled");

        ui.add_space(8.0);
        ui.label("How long to wait before and after each exposure with the build plate in place.");
        dragger(ui, "Rest Time", &mut self.rest_time, |x| {
            x.speed(0.1).suffix("s")
        });

        ui.add_space(8.0);
        ui.label("How many layers to apply the rest time to, after the bottom layers.");
        dragger(ui, "Rest Layers", &mut self.rest_layers, |x| {
            x.speed(1).suffix(" layers")
        });
    }

    fn post_slice(&self, _app: &App, goo: &mut GooFile) {
        goo.header.advance_mode = true;
        goo.header.exposure_delay_mode = true;

        // All bottom layers should have rest time added
        for layer in goo
            .layers
            .iter_mut()
            .take((goo.header.bottom_layers + self.rest_layers) as usize)
        {
            layer.before_lift_time = self.rest_time;
            layer.after_retract_time = self.rest_time;
        }
    }
}

pub fn get_plugin() -> Box<dyn Plugin> {
    Box::new(ElephantFootFixerPlugin {
        enabled: false,
        rest_time: 20.0,
        rest_layers: 20,
    })
}
