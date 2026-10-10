use egui::{Color32, Context, Frame, Id, Image, ImageSource, Modal, Widget, include_image};
use egui_phosphor::regular::INFO;

use crate::{VERSION, core::App, interface::components::button_row};

const LOGO: ImageSource = include_image!("../../../../dist/icon.png");
const BACKGROUND_TINT: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 100);

const DESCRIPTION: &str = "Welcome to mslicer — a high-performance, open-source slicer for MSLA resin printers, created by Connor Slade.";
const UPDATE_CHECK_TIP: &str =
    "You can disable or customize the check frequency in Workspace panel later.";

const GITHUB_LINK: &str = "https://github.com/connorslade/mslicer";
const HOMEPAGE_LINK: &str = "https://mslicer.com";
const GETTING_STARTED_LINK: &str = "https://mslicer.com/docs/getting-started";

pub fn ui(app: &mut App, ctx: &Context) {
    if !app.config.ui.about {
        return;
    }

    let modal = Modal::new(Id::new("about"))
        .backdrop_color(BACKGROUND_TINT)
        .frame(Frame::window(&ctx.style()))
        .show(ctx, |ui| {
            ui.set_width(400.0);

            ui.vertical_centered(|ui| {
                Image::new(LOGO).max_width(80.0).ui(ui);
                ui.heading(format!("mslicer v{VERSION}"));
            });
            ui.separator();

            ui.label(DESCRIPTION);
            ui.add_space(5.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                ui.label("Source code is available on Github at ");
                ui.hyperlink_to("@connorslade/mslicer", GITHUB_LINK)
                    .on_hover_text(GITHUB_LINK);
                ui.label(" and documentation is available at ");
                ui.hyperlink_to("mslicer.com", HOMEPAGE_LINK)
                    .on_hover_text(HOMEPAGE_LINK);
                ui.label(".");
            });

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                let mut check_enabled = app.config.ui.update_check.enabled();
                if ui
                    .checkbox(&mut check_enabled, "Check for Updates on Startup")
                    .changed()
                {
                    app.config.ui.update_check.toggle();
                }
                ui.label(INFO).on_hover_text(UPDATE_CHECK_TIP);
            });

            ui.add_space(5.0);
            button_row(
                ui,
                [
                    ("Continue", &mut || app.config.ui.about = false),
                    ("Getting Started Guide", &mut || {
                        let _ = open::that_detached(GETTING_STARTED_LINK);
                    }),
                ],
            );
        });

    if ctx.input(|i| !i.keys_down.is_empty()) || modal.should_close() {
        app.config.ui.about = false;
    }
}
