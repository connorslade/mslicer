use std::{fs::File, io::Write, mem, sync::Arc};

use const_format::concatcp;
use egui::{
    style::HandleShape, text::LayoutJob, Align, Button, Context, DragValue, FontSelection, Grid,
    Id, Layout, ProgressBar, RichText, Sense, Slider, Style, Ui, Vec2,
};
use egui_phosphor::regular::{FLOPPY_DISK_BACK, PAPER_PLANE_TILT};
use egui_wgpu::Callback;
use nalgebra::Vector2;
use rfd::FileDialog;
use wgpu::COPY_BUFFER_ALIGNMENT;

use crate::{
    app::{slice_operation::SliceResult, App},
    render::slice_preview::SlicePreviewRenderCallback,
    ui::{components::vec2_dragger, popup::Popup},
};
use common::serde::DynamicSerializer;

const FILENAME_POPUP_TEXT: &str =
    "To ensure the file name is unique, some extra random characters will be added on the end.";

pub fn ui(app: &mut App, ui: &mut Ui, ctx: &Context) {
    if let Some(slice_operation) = &app.slice_operation {
        let progress = &slice_operation.progress;

        let (current, total) = (progress.completed(), progress.total());

        if let Some(completion) = slice_operation.completion() {
            ui.horizontal(|ui| {
                ui.label(format!("Slicing completed in {completion}!"));

                ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(app.remote_print.is_initialized(), |ui| {
                            ui.menu_button(concatcp!(PAPER_PLANE_TILT, " Send to Printer"), |ui| {
                                let mqtt = app.remote_print.mqtt();
                                for printer in app.remote_print.printers().iter() {
                                    let client = mqtt.get_client(&printer.mainboard_id);

                                    let mut layout_job = LayoutJob::default();
                                    RichText::new(format!("{} ", client.attributes.name))
                                        .append_to(
                                            &mut layout_job,
                                            &Style::default(),
                                            FontSelection::Default,
                                            Align::LEFT,
                                        );
                                    RichText::new(&client.attributes.mainboard_id)
                                        .monospace()
                                        .append_to(
                                            &mut layout_job,
                                            &Style::default(),
                                            FontSelection::Default,
                                            Align::LEFT,
                                        );

                                    let result = app.slice_operation.as_ref().unwrap().result();
                                    let result = result.as_ref().unwrap();

                                    let mut serializer = DynamicSerializer::new();
                                    result.file.serialize(&mut serializer);
                                    let data = Arc::new(serializer.into_inner());

                                    let mainboard_id = printer.mainboard_id.clone();
                                    if ui.button(layout_job).clicked() {
                                        app.popup.open(name_popup(mainboard_id, data));
                                    }
                                }
                            });
                        });

                        if ui.button(concatcp!(FLOPPY_DISK_BACK, " Save")).clicked() {
                            let result = app.slice_operation.as_ref().unwrap().result();
                            let result = result.as_ref().unwrap();

                            if let Some(path) = FileDialog::new().save_file() {
                                let mut file = File::create(path).unwrap();
                                let mut serializer = DynamicSerializer::new();
                                result.file.serialize(&mut serializer);
                                file.write_all(&serializer.into_inner()).unwrap();
                            }
                        }
                    })
                });
            });

            let mut result = slice_operation.result();
            let Some(result) = result.as_mut() else {
                return;
            };

            let format = result.file.as_format();
            if !format.supports_preview() {
                ui.add_space(8.0);
                ui.label(format!(
                    "The {} format doesn't yet support previews...",
                    format.name()
                ));
            } else {
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        let layer_digits = result.layer_count.1 as usize;
                        ui.add(
                            DragValue::new(&mut result.slice_preview_layer)
                                .clamp_range(1..=result.file.info().layers)
                                .custom_formatter(|n, _| {
                                    format!("{:0>layer_digits$}/{}", n, result.layer_count.0)
                                }),
                        );
                        result.slice_preview_layer +=
                            ui.button(RichText::new("+").monospace()).clicked() as usize;
                        result.slice_preview_layer -=
                            ui.button(RichText::new("-").monospace()).clicked() as usize;

                        ui.separator();
                        ui.label("Offset");
                        vec2_dragger(ui, result.preview_offset.as_mut(), |x| x);

                        ui.separator();
                        ui.label("Scale");
                        ui.add(
                            DragValue::new(&mut result.preview_scale)
                                .clamp_range(0.1..=f32::MAX)
                                .speed(0.1),
                        );
                    });

                    slice_preview(ui, result);
                });
            }
        } else {
            ui.add(
                ProgressBar::new(current as f32 / total as f32)
                    .text(format!("{:.2}%", current as f32 / total as f32 * 100.0)),
            );

            ui.label(format!("Slicing... {current}/{total}"));
            ctx.request_repaint();
        }
    } else {
        ui.horizontal_wrapped(|ui| {
            ui.label("You can start a slice operation by pressing the");
            ui.code("Slice");
            ui.label("button on the top bar, or with the");
            ui.code("Ctrl+R");
            ui.label("keyboard shortcut.");
        });
    }
}

fn slice_preview(ui: &mut egui::Ui, result: &mut SliceResult) {
    let info = result.file.info();

    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.spacing_mut().slider_width = ui.available_size().y;
        ui.add(
            Slider::new(&mut result.slice_preview_layer, 1..=info.layers as usize)
                .vertical()
                .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
                .show_value(false),
        );

        let available_size = ui.available_size() - Vec2::new(5.0, 5.0);
        let (width, height) = (info.resolution.x, info.resolution.y);

        result.slice_preview_layer = result.slice_preview_layer.clamp(1, info.layers as usize);
        let new_preview = if result.last_preview_layer != result.slice_preview_layer {
            result.last_preview_layer = result.slice_preview_layer;

            let mut image =
                vec![0; ((width * height) as u64).next_multiple_of(COPY_BUFFER_ALIGNMENT) as usize];
            let layer = result.slice_preview_layer - 1;
            result.file.decode_layer(layer, &mut image);

            Some(image)
        } else {
            None
        };

        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let (rect, response) = ui
                .allocate_exact_size(Vec2::new(available_size.x, available_size.y), Sense::drag());

            let preview_scale = result.preview_scale.exp2();
            let drag = response.drag_delta();
            let aspect = rect.width() / rect.height() * height as f32 / width as f32;
            result.preview_offset.x -=
                drag.x / rect.width() * width as f32 / preview_scale * aspect;
            result.preview_offset.y += drag.y / rect.height() * height as f32 / preview_scale;

            if response.hovered() {
                let scroll = ui.input(|x| x.smooth_scroll_delta);
                result.preview_scale += scroll.y * 0.01;
                result.preview_scale = result.preview_scale.max(0.1);
            }

            let callback = Callback::new_paint_callback(
                rect,
                SlicePreviewRenderCallback {
                    dimensions: Vector2::new(info.resolution.x, info.resolution.y),
                    offset: result.preview_offset,
                    aspect: rect.width() / rect.height(),
                    scale: preview_scale,
                    new_preview,
                },
            );
            ui.painter().add(callback);
        });
    });
}

fn name_popup(mainboard_id: String, data: Arc<Vec<u8>>) -> Popup {
    Popup::new("Remote Send", move |app, ui| {
        ui.horizontal(|ui| {
            ui.label("File Name:");
            ui.text_edit_singleline(&mut app.state.working_filename);
        });

        ui.add_space(5.0);
        ui.label(FILENAME_POPUP_TEXT);
        ui.add_space(5.0);

        let spacing = ui.style().spacing.item_spacing.x;
        let width = (ui.available_size().x - spacing) / 2.0;
        let min_size = Vec2::new(width, 0.0);

        let mut close = false;
        let id = Id::new(&mainboard_id).with("remote_send");
        ui.centered_and_justified(|ui| {
            Grid::new(id)
                .min_col_width(width)
                .num_columns(2)
                .show(ui, |ui| {
                    close = ui.add(Button::new("Close").min_size(min_size)).clicked();
                    if ui.add(Button::new("Send").min_size(min_size)).clicked() {
                        close = true;
                        let name = mem::take(&mut app.state.working_filename)
                            .replace([' ', '/'], "_")
                            .replace("..", "");
                        app.remote_print
                            .upload(&mainboard_id, data.clone(), name)
                            .unwrap();
                    }
                });
        });

        close
    })
}
