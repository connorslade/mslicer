use egui::{
    Button, Color32, Context, Id, Image, ImageSource, LayerId, Order, Widget, Window,
    include_image, vec2,
};

use crate::app::App;

const DESCRIPTION: &str = "Welcome to mslicer — a high-performance, open-source slicer for MSLA resin printers, created by Connor Slade.";
const LOGO: ImageSource = include_image!("../../../dist/icon.png");
const BACKGROUND_TINT: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 100);

const GITHUB_LINK: &str = "https://github.com/connorslade/mslicer";
const HOMEPAGE_LINK: &str = "https://mslicer.com";
const GETTING_STARTED_LINK: &str = "https://mslicer.com/docs/getting-started";

pub fn ui(app: &mut App, ctx: &Context) {
    if !app.config.about {
        return;
    }

    let painter = ctx.layer_painter(LayerId::new(Order::Middle, Id::new("about")));
    painter.rect_filled(ctx.content_rect(), 0.0, BACKGROUND_TINT);

    let size = vec2(400.0, 227.0);
    let window = Window::new("about")
        .title_bar(false)
        .resizable(false)
        .fixed_size(size)
        .fixed_pos((ctx.content_rect().size() - size).to_pos2() / 2.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                Image::new(LOGO).max_width(80.0).ui(ui);
                ui.heading(concat!("mslicer v", env!("CARGO_PKG_VERSION")));
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
                let spacing = ui.style().spacing.item_spacing.x;
                let size = vec2(ui.available_size().x - spacing, 0.0) / 2.0;

                if Button::new("Close").min_size(size).ui(ui).clicked() {
                    app.config.about = false;
                }

                if Button::new("Getting Started Guide")
                    .min_size(size)
                    .ui(ui)
                    .clicked()
                {
                    let _ = open::that_detached(GETTING_STARTED_LINK);
                }
            });
        });

    if ctx.input(|i| i.pointer.any_down())
        && let Some(pos) = ctx.pointer_interact_pos()
        && !window.unwrap().response.rect.contains(pos)
    {
        app.config.about = false;
    }

    if ctx.input(|i| !i.keys_down.is_empty()) {
        app.config.about = false;
    }
}
