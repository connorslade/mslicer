use egui::{Context, Ui};

use crate::{app::App, ui::components::dragger};

use super::Plugin;

pub struct ElephantFootFixerPlugin {
    pub enabled: bool,
    pub rest_time: f32,
}

impl Plugin for ElephantFootFixerPlugin {
    fn name(&self) -> &'static str {
        "Elephant Foot Fixer"
    }

    fn ui(&mut self, _app: &mut App, ui: &mut Ui, _ctx: &Context) {
        ui.label("Fixes the 'Elephant Foot' issue by adding rest times before and after each bottom layer.");
        ui.checkbox(&mut self.enabled, "Enabled");
        dragger(ui, "Rest Time", &mut self.rest_time, |x| {
            x.speed(0.1).suffix("s")
        });
    }
}
