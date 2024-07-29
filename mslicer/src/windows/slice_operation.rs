use std::{fs::File, io::Write, mem, sync::Arc};

use egui::{
    style::HandleShape, text::LayoutJob, Align, Button, Context, DragValue, FontSelection, Grid,
    Id, Layout, ProgressBar, RichText, Sense, Slider, Style, Vec2, Window,
};
use egui_wgpu::Callback;
use goo_format::LayerDecoder;
use nalgebra::Vector2;
use rfd::FileDialog;

use crate::{
    app::App,
    render::slice_preview::SlicePreviewRenderCallback,
    slice_operation::SliceResult,
    ui::{components::vec2_dragger, popup::Popup},
};
use common::serde::DynamicSerializer;

const FILENAME_POPUP_TEXT: &str =
    "To ensure the file name is unique, some extra random characters will be added on the end.";

pub fn ui(app: &mut App, ctx: &Context) {
    let mut window_open = true;

    if let Some(slice_operation) = &app.slice_operation {
        let progress = &slice_operation.progress;

        let (current, total) = (progress.completed(), progress.total());

        let mut window = Window::new("Slice Operation");

        if current >= total {
            window = window.open(&mut window_open);
        }

        window.show(ctx, |ui| {
            let completion = slice_operation.completion();

            if completion.is_none() {
                ui.add(
                    ProgressBar::new(current as f32 / total as f32)
                        .text(format!("{:.2}%", current as f32 / total as f32 * 100.0)),
                );

                ui.label(format!("Slicing... {}/{}", current, total));
                ctx.request_repaint();
            } else {
                ui.horizontal(|ui| {
                    ui.label(format!("Slicing completed in {}!", completion.unwrap()));

                    ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                        ui.horizontal(|ui| {
                            ui.add_enabled_ui(app.remote_print.is_initialized(), |ui| {
                                ui.menu_button("Send to Printer", |ui| {
                                    let mqtt = app.remote_print.mqtt();
                                    for printer in app.remote_print.printers().iter() {
                                        let client = mqtt.get_client(&printer.mainboard_id);

                                        let mut layout_job = LayoutJob::default();
                                        RichText::new(&format!("{} ", client.attributes.name))
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
                                        result.goo.serialize(&mut serializer);
                                        let data = Arc::new(serializer.into_inner());

                                        let mainboard_id = printer.mainboard_id.clone();
                                        if ui.button(layout_job).clicked() {
                                            app.popup.open(name_popup(mainboard_id, data));
                                        }
                                    }
                                });
                            });

                            if ui.button("Save").clicked() {
                                let result = app.slice_operation.as_ref().unwrap().result();
                                let result = result.as_ref().unwrap();

                                if let Some(path) = FileDialog::new().save_file() {
                                    let mut file = File::create(path).unwrap();
                                    let mut serializer = DynamicSerializer::new();
                                    result.goo.serialize(&mut serializer);
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

                slice_preview(ui, result);

                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut result.slice_preview_layer)
                            .clamp_range(1..=result.goo.layers.len())
                            .custom_formatter(|n, _| format!("{}/{}", n, result.goo.layers.len())),
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
                    ui.add(DragValue::new(&mut result.preview_scale));
                });
            }
        });
    }

    if !window_open {
        app.slice_operation = None;
    }
}

fn slice_preview(ui: &mut egui::Ui, result: &mut SliceResult) {
    ui.horizontal(|ui| {
        ui.spacing_mut().slider_width = ui.available_size().x
            / result.goo.header.x_resolution as f32
            * result.goo.header.y_resolution as f32
            - 10.0;
        ui.add(
            Slider::new(&mut result.slice_preview_layer, 1..=result.goo.layers.len())
                .vertical()
                .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
                .show_value(false),
        );

        result.slice_preview_layer = result.slice_preview_layer.clamp(1, result.goo.layers.len());
        let new_preview = if result.last_preview_layer != result.slice_preview_layer {
            result.last_preview_layer = result.slice_preview_layer;
            let (width, height) = (
                result.goo.header.x_resolution as u32,
                result.goo.header.y_resolution as u32,
            );

            let layer_data = &result.goo.layers[result.slice_preview_layer - 1].data;
            let decoder = LayerDecoder::new(layer_data);

            let mut image = vec![0; (width * height) as usize];
            let mut pixel = 0;
            for run in decoder {
                for _ in 0..run.length {
                    image[pixel] = run.value;
                    pixel += 1;
                }
            }

            Some(image)
        } else {
            None
        };

        result.preview_scale = result.preview_scale.max(0.1);
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let available_size = ui.available_size();
            let (rect, _response) = ui.allocate_exact_size(
                Vec2::new(
                    available_size.x,
                    available_size.x / result.goo.header.x_resolution as f32
                        * result.goo.header.y_resolution as f32,
                ),
                Sense::drag(),
            );
            let callback = Callback::new_paint_callback(
                rect,
                SlicePreviewRenderCallback {
                    dimensions: Vector2::new(
                        result.goo.header.x_resolution as u32,
                        result.goo.header.y_resolution as u32,
                    ),
                    offset: result.preview_offset,
                    scale: result.preview_scale,
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
