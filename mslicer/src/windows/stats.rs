use egui::{Context, Ui};

use crate::app::App;

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.label(format!(
        "Frame Time: {:.2}ms",
        app.fps.frame_time() * 1000.0
    ));
    ui.label(format!("FPS: {:.2}", 1.0 / app.fps.frame_time()));
}
