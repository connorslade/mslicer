use egui::{Context, Ui};

use crate::{app::App, ui::markdown::CompiledMarkdown};

const DESCRIPTION: &str = "A work in progress FOSS slicer for resin printers, created by Connor Slade. Source code is available on Github at ";
const GITHUB_LINK: &str = "https://github.com/connorslade/mslicer";

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.monospace(concat!("mslicer v", env!("CARGO_PKG_VERSION")));
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(DESCRIPTION);
        ui.hyperlink_to("@connorslade/mslicer", GITHUB_LINK)
            .on_hover_text(GITHUB_LINK);
        ui.label(".");
    });

    ui.add_space(16.0);

    CompiledMarkdown::compile(include_str!("../../../docs/getting_started.md")).render(ui);

    ui.add_space(16.0);
    ui.heading("Stats");
    ui.label(format!(
        "Frame Time: {:.2}ms",
        app.fps.frame_time() * 1000.0
    ));
    ui.label(format!("FPS: {:.2}", 1.0 / app.fps.frame_time()));
}
