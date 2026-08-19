use egui::{Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Order, pos2};
use egui_phosphor::regular::{FILE_TEXT, FILES};

use crate::{app::App, system::arguments::OpenInto};

const HOVER_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 200);

pub fn update(app: &mut App, ctx: &Context) {
    let hovering = ctx.input(|x| x.raw.hovered_files.len());
    ctx.input(|x| {
        let mut open = OpenInto::default();
        for file in x.raw.dropped_files.iter() {
            if let Some(path) = &file.path {
                open.insert(path.to_owned());
            }
        }

        open.start(app);
    });

    if hovering > 0 {
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("drag_and_drop")));
        let rect = ctx.content_rect();
        let center = rect.center();

        painter.rect_filled(rect, 0.0, HOVER_BACKGROUND);

        painter.text(
            pos2(center.x, center.y - 54.0),
            Align2::CENTER_CENTER,
            if hovering == 1 { FILE_TEXT } else { FILES },
            FontId::new(64.0, FontFamily::Proportional),
            Color32::WHITE,
        );

        painter.text(
            center,
            Align2::CENTER_CENTER,
            "Drop files to import",
            FontId::default(),
            Color32::WHITE,
        );
    }
}
