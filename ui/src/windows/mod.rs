use eframe::Frame;
use egui::Context;

use crate::app::App;

mod about;
mod models;
mod slice_config;
mod slice_progress;
mod stats;
mod top_bar;
mod workspace;

pub struct Windows {
    pub show_about: bool,
    pub show_slice_config: bool,
    pub show_workspace: bool,
    pub show_models: bool,
    pub show_stats: bool,
}

pub fn ui(app: &mut App, ctx: &Context, frame: &mut Frame) {
    top_bar::ui(app, ctx, frame);
    slice_config::ui(app, ctx, frame);
    workspace::ui(app, ctx, frame);
    stats::ui(app, ctx, frame);
    models::ui(app, ctx, frame);
    about::ui(app, ctx, frame);
    slice_progress::ui(app, ctx, frame);
}

impl Default for Windows {
    fn default() -> Self {
        Self {
            show_about: false,
            show_slice_config: true,
            show_workspace: false,
            show_models: true,
            show_stats: false,
        }
    }
}
