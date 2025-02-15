use egui::{Context, Ui};

use crate::{
    app::App,
    ui::{markdown::CompiledMarkdown, state::DocsPage},
};

const DESCRIPTION: &str = "A work in progress FOSS slicer for resin printers, created by Connor Slade. Source code is available on Github at ";
const GITHUB_LINK: &str = "https://github.com/connorslade/mslicer";
const DOCS_LINK: &str = "https://github.com/connorslade/mslicer/tree/main/docs";

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

    ui.heading("Documentation");
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("This documentation is also available online ");
        ui.hyperlink_to("here", DOCS_LINK).on_hover_text(DOCS_LINK);
        ui.label(".");
    });

    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut app.state.docs_page,
            DocsPage::GettingStarted,
            "Getting Started",
        );
        ui.selectable_value(
            &mut app.state.docs_page,
            DocsPage::AnotherPage,
            "Another Page",
        );
    });

    CompiledMarkdown::compile(include_str!("../../../docs/getting_started.md")).render(ui);
}
