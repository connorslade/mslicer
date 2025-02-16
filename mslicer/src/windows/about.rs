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

    ui.add_space(8.0);

    let last_page = app.state.docs_page;
    ui.horizontal(|ui| {
        for page in DocsPage::ALL {
            ui.selectable_value(&mut app.state.docs_page, page, page.name());
        }
    });

    if last_page != app.state.docs_page || app.state.compiled_markdown.is_empty() {
        app.state.compiled_markdown = CompiledMarkdown::compile(app.state.docs_page.source());
    }

    ui.separator();

    app.state.compiled_markdown.render(ui);
}
