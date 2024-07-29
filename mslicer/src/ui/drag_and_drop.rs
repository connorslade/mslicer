use std::{
    borrow::Cow,
    fs::File,
    io::{BufReader, Cursor},
    path::Path,
};

use egui::{Align2, Color32, Context, FontId, Id, LayerId, Order};

use crate::app::App;

pub fn update(app: &mut App, ctx: &Context) {
    let is_hovering = ctx.input(|x| !x.raw.hovered_files.is_empty());
    ctx.input(|x| {
        for file in &x.raw.dropped_files {
            if let Some(path) = &file.path {
                let (name, format) = parse_path(&path);

                let file = File::open(&path).unwrap();
                let mut buf = BufReader::new(file);
                app.load_mesh(&mut buf, &format, name);
            } else if let Some(bytes) = &file.bytes {
                let (name, format) = parse_path(&file.path.as_ref().unwrap());
                let mut buf = Cursor::new(bytes);
                app.load_mesh(&mut buf, &format, name);
            }
        }
    });

    if is_hovering {
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("drag_and_drop")));
        let rect = ctx.screen_rect();
        painter.rect_filled(rect, 0.0, Color32::from_rgba_premultiplied(0, 0, 0, 200));

        let text = "Drop files to import";
        let font = FontId::default();
        let text_height = ctx.fonts(|x| x.row_height(&font));
        let text_pos = rect.center() - egui::vec2(0.0, text_height);
        painter.text(text_pos, Align2::CENTER_CENTER, text, font, Color32::WHITE);
    }
}

fn parse_path<'a>(path: &'a Path) -> (String, Cow<'a, str>) {
    let name = path.file_name().unwrap().to_str().unwrap().to_string();
    let ext = path.extension();
    let format = ext.unwrap_or_default().to_string_lossy();
    (name, format)
}
