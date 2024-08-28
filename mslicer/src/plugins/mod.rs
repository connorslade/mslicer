use egui::{Context, Ui};

use crate::app::App;

pub mod elephant_foot_fixer;

pub trait Plugin {
    fn name(&self) -> &'static str;
    fn ui(&mut self, app: &mut App, ui: &mut Ui, ctx: &Context);
}

pub struct PluginManager {
    pub plugins: Vec<Box<dyn Plugin>>,
}
