use egui::{Context, Ui};

use crate::app::App;

pub fn ui(_app: &mut App, ui: &mut Ui, _ctx: &Context) {
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
