use std::{fs::File, io::Write};

use common::serde::DynamicSerializer;
use eframe::Frame;
use egui::{Context, ProgressBar, Window};
use rfd::FileDialog;

use crate::app::App;

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    let mut window_open = true;
    let mut save_complete = false;

    if let Some(slice_operation) = &app.slice_operation {
        let progress = &slice_operation.progress;

        let (current, total) = (progress.completed(), progress.total());

        let mut window = Window::new("Slice Progress");

        if current >= total {
            window = window.open(&mut window_open);
        }

        window.show(ctx, |ui| {
            ui.add(
                ProgressBar::new(current as f32 / total as f32)
                    .text(format!("{:.2}%", current as f32 / total as f32 * 100.0)),
            );

            if current < total {
                ui.label(format!("Slicing... {}/{}", current, total));
                ctx.request_repaint();
            } else {
                ui.label("Slicing complete!");
                if ui.button("Save").clicked() {
                    let result = app.slice_operation.as_ref().unwrap().result();
                    let result = result.as_ref().unwrap();

                    if let Some(path) = FileDialog::new().save_file() {
                        let mut file = File::create(path).unwrap();
                        let mut serializer = DynamicSerializer::new();
                        result.goo.serialize(&mut serializer);
                        file.write_all(&serializer.into_inner()).unwrap();
                        save_complete = true;
                    }
                }
            }
        });
    }

    if !window_open || save_complete {
        app.slice_operation = None;
    }
}
