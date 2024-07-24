use eframe::Frame;
use egui::{Context, Ui, Window};
use egui_dock::TabViewer;

use crate::app::App;

use super::Tab;

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.monospace(concat!("mslicer v", env!("CARGO_PKG_VERSION")));
    ui.label("A work in progress FOSS slicer for resin printers. Created by Connor Slade.");
    ui.horizontal(|ui| {
        ui.label("Github:");
        ui.hyperlink_to(
            "@connorslade/mslicer",
            "https://github.com/connorslade/mslicer",
        );
    });
}
