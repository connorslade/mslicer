use eframe::Frame;
use egui::Context;

use crate::app::App;

mod about;
mod modals;
mod slice_config;
mod slice_progress;
mod stats;
mod top_bar;
mod transform;

pub struct Windows {
    pub show_about: bool,
    pub show_slice_config: bool,
    pub show_transform: bool,
    pub show_modals: bool,
    pub show_stats: bool,
}

pub fn ui(app: &mut App, ctx: &Context, frame: &mut Frame) {
    about::ui(app, ctx, frame);
    modals::ui(app, ctx, frame);
    slice_config::ui(app, ctx, frame);
    slice_progress::ui(app, ctx, frame);
    stats::ui(app, ctx, frame);
    top_bar::ui(app, ctx, frame);
    transform::ui(app, ctx, frame);
}

impl Default for Windows {
    fn default() -> Self {
        Self {
            show_about: false,
            show_slice_config: true,
            show_transform: true,
            show_modals: true,
            show_stats: false,
        }
    }
}