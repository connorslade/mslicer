use std::{f32, fs::File, io::Write, mem, sync::Arc};

use const_format::concatcp;
use egui::{
    Align, Button, Color32, Context, DragValue, FontSelection, Grid, Id, Layout, ProgressBar, Rect,
    RichText, Sense, Slider, StrokeKind, Style, Ui, Vec2, Widget, style::HandleShape,
    text::LayoutJob,
};
use egui_phosphor::regular::{
    CARET_DOWN, CARET_UP, CLOCK, CORNERS_IN, CROSSHAIR, DROP, FLOPPY_DISK_BACK, PAPER_PLANE_TILT,
};
use egui_wgpu::Callback;
use nalgebra::Vector2;

use crate::{
    app::{
        App,
        slice_operation::{ISLAND_COLOR, SliceResult},
        task::{FileDialog, IslandDetection, SaveResult},
    },
    render::slice_preview::SlicePreviewRenderCallback,
    ui::popup::Popup,
};
use common::{
    misc::human_duration, progress::Progress, serde::DynamicSerializer, slice::Format,
    units::Centimeter,
};

const FILENAME_POPUP_TEXT: &str =
    "To ensure the file name is unique, some extra random characters will be added on the end.";
const DETECT_ISLANDS_DESC: &str =
    "Will color disconnected chunks of voxels red in the slice preview.";

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    if let Some(slice_operation) = &app.slice_operation {
        let progress = &slice_operation.progress;

        if let Some(result) = slice_operation.result().as_mut() {
            let format = result.file.format();

            ui.horizontal(|ui| {
                ui.label(format!("Slicing completed in {}!", result.completion()));

                ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                    ui.horizontal(|ui| {
                        let enabled =
                            app.remote_print.is_initialized() && format.supports_preview();
                        ui.add_enabled_ui(enabled, |ui| {
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

                                    let mut serializer = DynamicSerializer::new();
                                    result.file.serialize(&mut serializer, Progress::new());
                                    let data = Arc::new(serializer.into_inner());

                                    let mainboard_id = printer.mainboard_id.clone();
                                    if ui.button(layout_job).clicked() {
                                        app.popup.open(name_popup(mainboard_id, data, format));
                                    }
                                }
                            });
                        });

                        if ui.button(concatcp!(FLOPPY_DISK_BACK, " Save")).clicked() {
                            let file = result.file.clone();
                            let task = FileDialog::save_file(
                                (format.name(), &[format.extension()]),
                                move |_app, path, tasks| {
                                    let path = path.with_extension(format.extension());
                                    let file_name = path.file_name().unwrap().to_string_lossy();
                                    let mut out = File::create(&path).unwrap();

                                    tasks.push(Box::new(SaveResult::new(
                                        (file, file_name.into_owned()),
                                        move |bytes| out.write_all(&bytes).unwrap(),
                                    )));
                                },
                            );
                            app.tasks.add(task);
                        }

                        ui.separator();
                        ui.add_enabled_ui(!result.detected_islands, |ui| {
                            if ui
                                .button(concatcp!(CROSSHAIR, " Detect Islands"))
                                .on_hover_text(DETECT_ISLANDS_DESC)
                                .clicked()
                            {
                                result.detected_islands = true;
                                app.tasks.add(IslandDetection::new(
                                    result.file.clone(),
                                    result.annotations.clone(),
                                ));
                            }
                            slice_preview
                        });
                    })
                });
            });

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
                                .range(1..=result.file.info().layers)
                                .custom_formatter(|n, _| {
                                    format!("{:0>layer_digits$}/{}", n, result.layer_count.0)
                                }),
                        );
                        result.slice_preview_layer +=
                            ui.button(RichText::new(CARET_UP)).clicked() as usize;
                        result.slice_preview_layer -=
                            ui.button(RichText::new(CARET_DOWN)).clicked() as usize;

                        ui.separator();
                        if ui.button(concatcp!(CORNERS_IN, " Reset View")).clicked() {
                            result.preview_offset = Vector2::zeros();
                            result.preview_scale = 1.0;
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let duration = human_duration(result.print_time.convert());
                            ui.label(format!("{CLOCK} {duration}"));

                            ui.separator();
                            let volume = result.volume.get::<Centimeter>(); // cm³ = ml
                            ui.label(format!("{DROP} {volume:.2} ml"));

                            ui.add_space(ui.available_width());
                        })
                    });

                    slice_preview(ui, result);
                });
            }
        } else {
            Grid::new("slice_operation")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Slicing");
                    ui.add(ProgressBar::new(progress.progress()).show_percentage());
                    ui.end_row();

                    let post_process = &slice_operation.post_processing_progress;
                    for i in 0..post_process.count() {
                        let progress = post_process[i].progress();
                        let name = ["Elephant Foot Fixer", "Anti Aliasing"][i];
                        if progress > 0.0 {
                            ui.label(name);
                            ui.add(ProgressBar::new(progress).show_percentage());
                            ui.end_row();
                        }
                    }
                });
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
        layer_slider(ui, result);

        let available_size = ui.available_size() - Vec2::new(5.0, 5.0);
        let (width, height) = (info.resolution.x, info.resolution.y);

        result.slice_preview_layer = result.slice_preview_layer.clamp(1, info.layers as usize);
        let new_preview = if result.last_preview_layer != result.slice_preview_layer {
            result.last_preview_layer = result.slice_preview_layer;
            let size = (width * height) as usize;

            let mut layer = vec![0u8; size];
            let layer_idx = result.slice_preview_layer - 1;
            result.file.decode_layer(layer_idx, &mut layer);

            let mut layer_annotations = vec![0u8; size];
            (result.annotations.lock()).decode_layer(layer_idx, &mut layer_annotations);

            Some((layer, layer_annotations))
        } else {
            None
        };

        egui::Frame::canvas(ui.style())
            .fill(ui.style().visuals.panel_fill)
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(available_size.x, available_size.y),
                    Sense::drag(),
                );

                let drag = response.drag_delta();
                let aspect = rect.width() / rect.height() * height as f32 / width as f32;
                let preview_scale = result.preview_scale.powi(2);
                result.preview_offset.x -=
                    drag.x / rect.width() * width as f32 / preview_scale * aspect;
                result.preview_offset.y += drag.y / rect.height() * height as f32 / preview_scale;

                if let Some(pointer) = response.hover_pos()
                    && rect.contains(pointer)
                {
                    let dimensions = Vec2::new(info.resolution.x as f32, info.resolution.y as f32);
                    let aspect = rect.width() / rect.height() * dimensions.y / dimensions.x;

                    let scroll = ui.input(|x| x.smooth_scroll_delta);
                    result.preview_scale =
                        (result.preview_scale + scroll.y * 0.01).clamp(0.5, 10.0);

                    if scroll.y != 0.0 {
                        // Scale around the cursor, not the center of the layer
                        let t = (pointer - rect.min) / (rect.max - rect.min) - Vec2::splat(0.5);
                        let delta = (t * Vec2::new(aspect, 1.0) * dimensions)
                            * (preview_scale.recip() - result.preview_scale.powi(-2));
                        result.preview_offset.x += delta.x;
                        result.preview_offset.y -= delta.y;
                    }
                }

                let callback = Callback::new_paint_callback(
                    rect,
                    SlicePreviewRenderCallback {
                        dimensions: Vector2::new(info.resolution.x, info.resolution.y),
                        offset: result.preview_offset,
                        aspect: rect.width() / rect.height(),
                        scale: result.preview_scale.powi(2),
                        new_preview,
                    },
                );
                ui.painter().add(callback);
            });
    });
}

fn layer_slider(ui: &mut egui::Ui, result: &mut SliceResult) {
    let info = result.file.info();

    ui.spacing_mut().slider_width = ui.available_size().y;
    let slider = Slider::new(&mut result.slice_preview_layer, 1..=info.layers as usize)
        .vertical()
        .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
        .show_value(false)
        .ui(ui);

    let painter = ui.painter_at(slider.rect);
    let slice = slider.rect.height() / info.layers as f32;

    let visuals = ui.style().interact(&slider);
    let rail = ui.spacing().slider_rail_height;
    let handle_r = slider.rect.width() / 2.5;
    let height = slider.rect.height() - 2.0 * handle_r;
    let pos = |t: f32| slider.rect.center_bottom() - Vec2::Y * (handle_r + height * t);

    let slider_t = (result.slice_preview_layer - 1) as f32 / (info.layers - 1) as f32;
    let handle_inner_r = handle_r - visuals.fg_stroke.width;
    let handle_t = (handle_inner_r + visuals.expansion) / height;

    let annotations = result.annotations.lock();
    for i in 0..info.layers {
        if annotations.contains(i as usize) {
            let t = i as f32 / (info.layers.saturating_sub(1)) as f32;
            let width = if (slider_t - t).abs() < handle_t {
                handle_inner_r * 2.0 + visuals.expansion
            } else {
                rail
            };

            let rect = Rect::from_center_size(pos(t), Vec2::new(width, slice * 2.0));
            painter.rect_filled(rect, 0, ISLAND_COLOR);
        }
    }
    drop(annotations);

    let rect = Rect::from_center_size(
        pos(slider_t),
        2.0 * Vec2::splat(handle_r + visuals.expansion),
    );
    painter.rect(
        rect,
        visuals.corner_radius,
        Color32::TRANSPARENT,
        visuals.fg_stroke,
        StrokeKind::Inside,
    );
}

fn name_popup(mainboard_id: String, data: Arc<Vec<u8>>, format: Format) -> Popup {
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
                            .upload(&mainboard_id, data.clone(), name, format)
                            .unwrap();
                    }
                });
        });

        close
    })
}
