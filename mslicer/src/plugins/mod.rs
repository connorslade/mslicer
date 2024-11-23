use egui::{Context, Ui};
use slicer::format::FormatSliceFile;

use crate::app::App;

pub mod anti_alias;
pub mod elephant_foot_fixer;

pub trait Plugin {
    fn name(&self) -> &'static str;
    fn ui(&mut self, app: &mut App, ui: &mut Ui, ctx: &Context);

    fn post_slice(&self, _app: &App, _goo: &mut FormatSliceFile) {}
}

pub struct PluginManager {
    pub plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn post_slice(&self, app: &App, file: &mut FormatSliceFile) {
        for plugin in &self.plugins {
            plugin.post_slice(app, file);
        }
    }
}
