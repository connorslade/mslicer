use egui::{Context, Ui};

use crate::app::App;

const DESCRIPTION: &str = "A work in progress FOSS slicer for resin printers, created by Connor Slade. Source code is available on Github at ";

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.monospace(concat!("mslicer v", env!("CARGO_PKG_VERSION")));
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(DESCRIPTION);
        ui.hyperlink_to(
            "@connorslade/mslicer",
            "https://github.com/connorslade/mslicer",
        );
        ui.label(".");
    });

    ui.add_space(16.0);
    ui.heading("Getting Started");

    ui.label(include_str!("../../../docs/getting_started.txt"));

    ui.add_space(16.0);
    ui.heading("Stats");
    ui.label(format!(
        "Frame Time: {:.2}ms",
        app.fps.frame_time() * 1000.0
    ));
    ui.label(format!("FPS: {:.2}", 1.0 / app.fps.frame_time()));
}
