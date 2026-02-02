use std::{borrow::Cow, fs::File, path::Path};

use egui::{Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Order, pos2};
use egui_phosphor::regular::{FILE_TEXT, FILES};
use itertools::Itertools;

use crate::app::{
    App,
    task::{MeshLoad, ProjectLoad},
};

const HOVER_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 200);

pub fn update(app: &mut App, ctx: &Context) {
    let hovering = ctx.input(|x| x.raw.hovered_files.len());
    ctx.input(|x| {
        for (file, (name, format)) in x
            .raw
            .dropped_files
            .iter()
            .map(|x| (x, parse_path(x.path.as_ref().unwrap())))
            .sorted_by_key(|(_, (_, format))| format == "mslicer")
        {
            if let Some(path) = &file.path {
                if format == "mslicer" {
                    app.tasks.add(ProjectLoad::new(path.to_path_buf()));
                } else {
                    let file = File::open(path).unwrap();
                    app.tasks.add(MeshLoad::file(file, name, format.into()));
                }
            }
        }
    });

    if hovering > 0 {
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("drag_and_drop")));
        let rect = ctx.content_rect();
        let center = rect.center();

        painter.rect_filled(rect, 0.0, HOVER_BACKGROUND);

        let icon = if hovering == 1 { FILE_TEXT } else { FILES };
        let text = "Drop files to import";

        painter.text(
            pos2(center.x, center.y - 54.0),
            Align2::CENTER_CENTER,
            icon,
            FontId::new(64.0, FontFamily::Proportional),
            Color32::WHITE,
        );

        painter.text(
            center,
            Align2::CENTER_CENTER,
            text,
            FontId::default(),
            Color32::WHITE,
        );
    }
}

fn parse_path(path: &Path) -> (String, Cow<'_, str>) {
    let name = path.file_name().unwrap().to_str().unwrap().to_string();
    let ext = path.extension();
    let format = ext.unwrap_or_default().to_string_lossy();
    (name, format)
}
