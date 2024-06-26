use eframe::Frame;
use egui::{Context, Window};

use crate::app::App;

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    Window::new("Stats")
        .open(&mut app.windows.show_stats)
        .show(ctx, |ui| {
            ui.label(format!(
                "Frame Time: {:.2}ms",
                app.fps.frame_time() * 1000.0
            ));
            ui.label(format!("FPS: {:.2}", 1.0 / app.fps.frame_time()));
        });
}
