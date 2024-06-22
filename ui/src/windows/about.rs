use eframe::Frame;
use egui::{Context, Window};

use crate::app::App;

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    Window::new("About")
        .open(&mut app.windows.show_about)
        .show(ctx, |ui| {
            ui.monospace(concat!("mslicer v", env!("CARGO_PKG_VERSION")));
            ui.label("A work in progress FOSS slicer for resin printers. Created by Connor Slade.");
            ui.horizontal(|ui| {
                ui.label("Github:");
                ui.hyperlink_to(
                    "@connorslade/mslicer",
                    "https://github.com/connorslade/mslicer",
                );
            });
        });
}
