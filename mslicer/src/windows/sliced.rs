// todo: split this into multiple files

use std::{
    f32,
    fs::File,
    io::{BufReader, Write},
    mem,
    sync::Arc,
};

use const_format::concatcp;
use egui::{
    Align, Align2, Button, CollapsingHeader, Color32, ComboBox, Context, DragValue, FontId,
    FontSelection, Frame, Grid, Id, ImageSource, Layout, ProgressBar, Rect, RichText, ScrollArea,
    Sense, SidePanel, Slider, StrokeKind, Style, Ui, Vec2, Widget, load::SizedTexture, panel::Side,
    style::HandleShape, text::LayoutJob, vec2,
};
use egui_phosphor::regular::{
    CAMERA, CARET_DOWN, CARET_UP, CLOCK, CORNERS_IN, CROSSHAIR, CUBE_TRANSPARENT, DROP,
    FLOPPY_DISK_BACK, PAPER_PLANE_TILT, SIDEBAR, SWAP, TEXT_AA,
};
use egui_plot::{Line, LineStyle, Plot, VLine};
use egui_wgpu::Callback;
use epaint_default_fonts::UBUNTU_LIGHT;
use image::{ImageFormat, Rgba, RgbaImage, imageops::FilterType};
use nalgebra::Vector2;

use crate::{
    app::{
        App,
        config::{
            Config,
            sliced::{SlicePreviewCoordinateSpace, SlicePreviewView, SlicedConfig},
        },
        slice_operation::{
            GenericSliceData, GenericSliceResult, ISLAND_COLOR, PreviewImage, RasterSliceResult,
            SliceOperation, SliceResult,
        },
    },
    render::slice_preview::SlicePreviewRenderCallback,
    task::{FileDialog, IslandDetection, ReconstructMesh, SaveResult, TaskManager},
    ui::{
        components::{collapsing_toggle, grid},
        management::{LazyText, LazyTextureId},
        popup::{Popup, PopupManager},
        state::UiState,
    },
    windows::slice_config::exposure_config,
};
use common::{
    misc::{IMAGE_FORMATS, human_duration},
    progress::Progress,
    serde::DynamicSerializer,
    slice::{
        SliceConfig, SliceMode,
        format::{Format, RasterFormat},
    },
    units::{Centimeter, Milimeter},
};

const FILENAME_POPUP_TEXT: &str =
    "To ensure the file name is unique, some extra random characters will be added on the end.";
const DETECT_ISLANDS_DESC: &str =
    "Will color disconnected chunks of voxels red in the slice preview.";
const SURFACE_AREA_DESC: &str = "Surface area in cm² of each layer. Layers with higher areas will adhere more to the FEP potentially causing print failures.";

pub fn ui(app: &mut App, ui: &mut Ui, ctx: &Context) {
    if let Some(slice_operation) = &app.slice_operation {
        let progress = &slice_operation.progress;

        if let Some(result) = slice_operation.result().as_mut() {
            let format = app.project.slice_config.mode;

            if mem::take(&mut result.fresh) {
                app.state.preview_layer = 1;
                app.state.last_preview_layer = 0;
                app.state.preview_offset = Vector2::zeros();
                app.state.preview_scale = 1.0;

                let layers = result.inner.layers();
                app.state.layer_count = (layers, layers.to_string().len() as u8);
            }

            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} completed in {}!",
                    ["Loading", "Slicing"][result.sliced as usize],
                    result.completion()
                ));

                ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                    ui.horizontal(|ui| {
                        sidebar_button(&mut app.config.sliced, ui);
                        ui.separator();

                        let enabled =
                            app.remote_print.is_initialized() && format == SliceMode::Raster;
                        ui.add_enabled_ui(enabled, |ui| {
                            ui.menu_button(concatcp!(PAPER_PLANE_TILT, " Send to Printer"), |ui| {
                                for client in app.remote_print.clients().iter() {
                                    let mut layout_job = LayoutJob::default();
                                    RichText::new(format!("{} ", client.name)).append_to(
                                        &mut layout_job,
                                        &Style::default(),
                                        FontSelection::Default,
                                        Align::LEFT,
                                    );
                                    RichText::new(&client.mainboard).monospace().append_to(
                                        &mut layout_job,
                                        &Style::default(),
                                        FontSelection::Default,
                                        Align::LEFT,
                                    );

                                    if ui.button(layout_job).clicked() {
                                        let file = result.slice_data().file(
                                            &result.config,
                                            &slice_operation.preview(),
                                            RasterFormat::Ctb.into(),
                                        );

                                        let mut serializer = DynamicSerializer::new();
                                        file.serialize(&mut serializer, Progress::new());
                                        let data = Arc::new(serializer.into_inner());

                                        app.popup.open(name_popup(client.mainboard.clone(), data));
                                    }
                                }
                            });
                        });

                        ui.menu_button(concatcp!(FLOPPY_DISK_BACK, " Save"), |ui| {
                            let formats: &[Format] = match result.config.mode {
                                SliceMode::Raster => &Format::RASTER,
                                SliceMode::Vector => &Format::VECTOR,
                            };

                            ui.set_width(150.0);
                            for &format in formats {
                                let disabled = result.variable_layer_height
                                    && matches!(format, Format::Raster(RasterFormat::NanoDLP));

                                if ui
                                    .add_enabled(!disabled, Button::new(format.name()))
                                    .clicked()
                                {
                                    app.tasks.add(save_file(
                                        result.config.clone(),
                                        slice_operation.preview(),
                                        format,
                                        result.slice_data(),
                                    ));
                                }
                            }
                        });

                        ui.separator();

                        // todo: these functions shouldn't available if slice is
                        // not raster
                        let can_detect = (result.inner.as_raster())
                            .map(|x| !x.detected_islands)
                            .unwrap_or_default();
                        ui.add_enabled_ui(can_detect, |ui| {
                            if ui
                                .button(concatcp!(CROSSHAIR, " Detect Islands"))
                                .on_hover_text(DETECT_ISLANDS_DESC)
                                .clicked()
                                && let GenericSliceResult::Raster(raster) = &mut result.inner
                            {
                                raster.detected_islands = true;
                                app.tasks.add(IslandDetection::new(
                                    result.config.platform_resolution,
                                    raster.layers.clone(),
                                    raster.annotations.clone(),
                                ));
                            }
                            slice_preview
                        });

                        if let GenericSliceResult::Raster(raster) = &mut result.inner {
                            ui.menu_button(
                                concatcp!(CUBE_TRANSPARENT, " Reconstruct Mesh"),
                                |ui| {
                                    for (name, subsample) in
                                        [("Corse", 20), ("Fine", 10), ("Exact", 1)]
                                    {
                                        if ui.button(format!("{name} ({subsample})")).clicked() {
                                            app.tasks.add(ReconstructMesh::new(
                                                result.config.clone(),
                                                raster.layers.clone(),
                                                subsample,
                                            ));
                                        }
                                    }
                                },
                            );
                        }
                    })
                });
            });

            SidePanel::new(Side::Right, "sidebar")
                .resizable(false)
                .show_animated_inside(ui, app.config.sliced.sidebar, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        sidebar(
                            slice_operation,
                            result,
                            &mut app.state,
                            &mut app.config,
                            &mut app.tasks,
                            &mut app.popup,
                            ui,
                            ctx,
                        );
                    })
                });

            match &mut result.inner {
                GenericSliceResult::Raster(raster) => {
                    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                        let state = &mut app.state;
                        ui.horizontal(|ui| {
                            let layer_digits = state.layer_count.1 as usize;
                            DragValue::new(&mut state.preview_layer)
                                .range(1..=state.layer_count.0)
                                .custom_formatter(|n, _| {
                                    format!("{:0>layer_digits$}/{}", n, state.layer_count.0)
                                })
                                .ui(ui);
                            state.preview_layer += ui.button(CARET_UP).clicked() as usize;
                            state.preview_layer -= ui.button(CARET_DOWN).clicked() as usize;

                            ui.separator();
                            if ui.button(concatcp!(CORNERS_IN, " Reset View")).clicked() {
                                state.preview_offset = Vector2::zeros();
                                state.preview_scale = 1.0;
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let duration = human_duration(raster.print_time.convert());
                                ui.label(format!("{CLOCK} {duration}"));

                                ui.separator();
                                let volume = raster.volume.get::<Centimeter>(); // cm³ = ml
                                ui.label(format!("{DROP} {volume:.2} ml"));

                                ui.take_available_width();
                            })
                        });

                        // Some printers have non square pixels, this option
                        // allows you to select between seeing each pixel as a
                        // square or as the actual shape on the printer.
                        let platform = result.config.platform_resolution;
                        let pixel_aspect = match app.config.sliced.coordinate_space {
                            SlicePreviewCoordinateSpace::ScreenSpace => 1.0,
                            SlicePreviewCoordinateSpace::WorldSpace => {
                                let platform_size = (result.config.platform_size.xy())
                                    .map(|x| x.get::<Milimeter>());
                                let aspect = platform.cast::<f32>().component_div(&platform_size);
                                (aspect.y / aspect.x).recip()
                            }
                        };

                        let flip = matches!(app.config.sliced.view, SlicePreviewView::Screen);
                        let multisample = app.config.sliced.multisample;
                        slice_preview(state, ui, raster, platform, pixel_aspect, flip, multisample);
                    });
                }
                GenericSliceResult::Vector(_) => {
                    ui.add_space(8.0);
                    ui.label("Vector formats don't yet support previews…");
                }
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
                        let name = ["Elephant Foot Fixer", "Variable Layer Heights"][i];
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

fn slice_preview(
    state: &mut UiState,
    ui: &mut egui::Ui,
    result: &mut RasterSliceResult,
    platform: Vector2<u32>,
    pixel_aspect: f32,
    flip: bool,
    multisample: u32,
) {
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        layer_slider(state, ui, result);

        let available_size = ui.available_size() - Vec2::new(5.0, 5.0);
        let [width, height] = *platform.as_ref();

        Frame::canvas(ui.style())
            .fill(ui.style().visuals.panel_fill)
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(available_size.x, available_size.y),
                    Sense::drag(),
                );

                let drag = response.drag_delta();
                let aspect = rect.width() / rect.height() * height as f32 / width as f32;
                let preview_scale = state.preview_scale.powi(2);
                state.preview_offset.x -=
                    drag.x / rect.width() * width as f32 / preview_scale * aspect;
                state.preview_offset.y += drag.y / rect.height() * height as f32 / preview_scale;

                if let Some(pointer) = response.hover_pos()
                    && rect.contains(pointer)
                {
                    let dimensions = Vec2::new(width as f32, height as f32);
                    let aspect = rect.width() / rect.height() * dimensions.y / dimensions.x;

                    let scroll = ui.input(|x| x.smooth_scroll_delta);
                    state.preview_scale = (state.preview_scale + scroll.y * 0.01).clamp(0.5, 10.0);

                    if scroll.y != 0.0 {
                        // Scale around the cursor, not the center of the layer
                        let t = (pointer - rect.min) / (rect.max - rect.min) - Vec2::splat(0.5);
                        let delta = (t * Vec2::new(aspect, 1.0) * dimensions)
                            * (preview_scale.recip() - state.preview_scale.powi(-2));
                        state.preview_offset.x += delta.x;
                        state.preview_offset.y -= delta.y;
                    }
                }

                let mut scale = Vector2::repeat(state.preview_scale.powi(2));
                flip.then(|| scale.y *= -1.0);

                state.preview_layer = state.preview_layer.clamp(1, state.layer_count.0);
                let new_preview = if ui.is_rect_visible(rect)
                    && (state.last_preview_layer != state.preview_layer
                        || result.annotations.take_updated())
                {
                    state.last_preview_layer = state.preview_layer;

                    let layer_idx = state.preview_layer - 1;
                    let layer = result.layers[layer_idx].data.clone();
                    let annotations = result.annotations.lock().get_layer(layer_idx);

                    Some((layer, annotations))
                } else {
                    None
                };

                let callback = Callback::new_paint_callback(
                    rect,
                    SlicePreviewRenderCallback {
                        dimensions: platform,
                        offset: state.preview_offset,
                        aspect: rect.width() / rect.height(),
                        pixel_aspect,
                        scale,
                        new_preview,
                        multisample,
                    },
                );
                ui.painter().add(callback);
            });
    });
}

fn layer_slider(state: &mut UiState, ui: &mut egui::Ui, result: &mut RasterSliceResult) {
    ui.spacing_mut().slider_width = ui.available_size().y;

    let layer_count = state.layer_count.0;
    let slider = Slider::new(&mut state.preview_layer, 1..=layer_count)
        .vertical()
        .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
        .show_value(false)
        .ui(ui);

    let painter = ui.painter_at(slider.rect);
    let slice = slider.rect.height() / layer_count as f32;

    let visuals = ui.style().interact(&slider);
    let rail = ui.spacing().slider_rail_height;
    let handle_r = slider.rect.width() / 2.5;
    let height = slider.rect.height() - 2.0 * handle_r;
    let pos = |t: f32| slider.rect.center_bottom() - Vec2::Y * (handle_r + height * t);

    let slider_t = (state.preview_layer - 1) as f32 / (layer_count - 1) as f32;
    let handle_inner_r = handle_r - visuals.fg_stroke.width;
    let handle_t = (handle_inner_r + visuals.expansion) / height;

    let annotations = result.annotations.lock();
    for i in 0..layer_count {
        if annotations.contains(i) {
            let t = i as f32 / (layer_count.saturating_sub(1)) as f32;
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
        let id = Id::new(&mainboard_id).with("remote_print");
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
                            .upload(&mainboard_id, data.clone(), name, RasterFormat::Ctb)
                            .unwrap();
                    }
                });
        });

        close
    })
}

fn save_file(
    config: SliceConfig,
    preview_image: Arc<RgbaImage>,
    format: Format,
    data: GenericSliceData,
) -> FileDialog {
    FileDialog::save_file(
        (format.name(), &[format.extension()]),
        move |_app, path, tasks| {
            let path = path.with_extension(format.extension());
            let file_name = path.file_name().unwrap().to_string_lossy();
            let mut out = File::create(&path).unwrap();

            let file = data.file(&config, &preview_image, format);
            tasks.push(Box::new(SaveResult::new(
                (file, file_name.into_owned()),
                move |bytes| out.write_all(&bytes).unwrap(),
            )));
        },
    )
}

fn sidebar_button(sliced: &mut SlicedConfig, ui: &mut Ui) {
    let y = ui.spacing().interact_size.y;
    let (rect, mut response) = ui.allocate_exact_size(vec2(y, y), egui::Sense::click());
    response = response.on_hover_text("Toggle sidebar visibility.");
    sliced.sidebar ^= response.clicked();

    let visuals = ui.style().interact_selectable(&response, sliced.sidebar);
    ui.painter().rect(
        rect,
        visuals.corner_radius,
        visuals.bg_fill,
        visuals.bg_stroke,
        StrokeKind::Outside,
    );

    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        SIDEBAR,
        FontId::default(),
        visuals.text_color(),
    );
}

// todo: do smth abt these args sob
fn sidebar(
    operation: &SliceOperation,
    result: &mut SliceResult,
    state: &mut UiState,
    config: &mut Config,
    tasks: &mut TaskManager,
    popups: &mut PopupManager,
    ui: &mut Ui,
    ctx: &Context,
) {
    CollapsingHeader::new("Preview Image")
        .default_open(true)
        .show(ui, |ui| {
            let mut previews = operation.previews.lock();
            let preview = previews.as_mut().unwrap();
            let mut reset_preview = false;

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                reset_preview = ui.button(concatcp!(CAMERA, " Retake")).clicked();

                if ui.button(concatcp!(SWAP, " Replace")).clicked() {
                    let task =
                        FileDialog::pick_file(("Image", &IMAGE_FORMATS), |app, path, _tasks| {
                            let format = ImageFormat::from_path(path).unwrap();
                            let file = BufReader::new(File::open(path).unwrap());
                            let mut image = image::load(file, format).unwrap();

                            if image.width() > 512 || image.height() > 512 {
                                image = image.resize(512, 512, FilterType::Triangle);
                            }

                            if let Some(operation) = app.slice_operation.as_mut() {
                                operation.add_preview(image.to_rgba8());
                            }
                        });
                    tasks.add(task);
                }

                if ui.button(concatcp!(TEXT_AA, " Add Text")).clicked() {
                    let mut text = String::default();
                    let mut size = 10.0;
                    popups.open(Popup::new("Add Text", move |app, ui| {
                        grid("text").show(ui, |ui| {
                            ui.label("Text");
                            ui.text_edit_singleline(&mut text);
                            ui.end_row();

                            ui.label("Size");
                            ui.horizontal(|ui| {
                                DragValue::new(&mut size).suffix("%").ui(ui);
                                ui.take_available_width();
                            });
                            ui.end_row();
                        });

                        let mut close = false;
                        ui.centered_and_justified(|ui| {
                            if ui.button("Composite").clicked() {
                                if let Some(slice_operation) = app.slice_operation.as_mut()
                                    && let Some(image) = slice_operation.previews.lock().as_mut()
                                {
                                    // todo: dont keep re-parsing
                                    let font =
                                        ab_glyph::FontRef::try_from_slice(UBUNTU_LIGHT).unwrap();
                                    let size = image.image.height() as f32 * size / 100.0;
                                    let color = Rgba([255, 255, 255, 255]);
                                    let new = imageproc::drawing::draw_text(
                                        &*image.image,
                                        color,
                                        0,
                                        0,
                                        size,
                                        &font,
                                        &text,
                                    );

                                    *image = PreviewImage {
                                        image: Arc::new(new),
                                        texture: LazyTextureId::empty(),
                                    };
                                }

                                close = true;
                            }
                        });

                        close
                    }));
                }
            });

            let available = ui.available_width();
            let (width, height) = (preview.image.width(), preview.image.height());

            let size = vec2(available, available / width as f32 * height as f32);
            let texture = SizedTexture::new(preview.texture.get(ctx, &preview.image), size);

            reset_preview.then(|| previews.take());

            ui.add_space(4.0);
            ui.image(ImageSource::Texture(texture))
                .on_hover_text(LazyText::new(move || format!("{width}×{height}")))
        });

    CollapsingHeader::new("Slice Preview")
        .default_open(true)
        .show(ui, |ui| {
            let sliced = &mut config.sliced;
            grid("slice_preview").show(ui, |ui| {
                ui.label("Coordinate Space");
                ComboBox::from_id_salt("coordinate_space")
                    .selected_text(sliced.coordinate_space.name())
                    .show_ui(ui, |ui| {
                        for mode in SlicePreviewCoordinateSpace::ALL {
                            ui.selectable_value(&mut sliced.coordinate_space, *mode, mode.name());
                        }
                    });
                ui.end_row();

                ui.label("View Direction");
                ComboBox::from_id_salt("view")
                    .selected_text(sliced.view.name())
                    .show_ui(ui, |ui| {
                        for view in SlicePreviewView::ALL {
                            ui.selectable_value(&mut sliced.view, *view, view.name());
                        }
                    });
                ui.end_row();

                ui.label("Anti-Aliasing");
                ui.horizontal(|ui| {
                    DragValue::new(&mut sliced.multisample)
                        .range(1..=64)
                        .suffix("×")
                        .ui(ui);
                    ui.take_available_width();
                });
                ui.end_row();
            });
        });

    ui.add_space(8.0);
    ui.heading("Exposure");

    let mut exposure_changed = false;
    CollapsingHeader::new("Config").show(ui, |ui| {
        grid("exposure").show(ui, |ui| {
            ui.label("First Layers");
            exposure_changed |= DragValue::new(&mut result.config.first_layers)
                .ui(ui)
                .changed();
            ui.end_row();

            ui.label("Transition Layers");
            ui.horizontal(|ui| {
                exposure_changed |= DragValue::new(&mut result.config.transition_layers)
                    .ui(ui)
                    .changed();
                ui.take_available_width();
            });
            ui.end_row();
        });
    });

    CollapsingHeader::new("Normal Layers").show(ui, |ui| {
        exposure_changed |= exposure_config(ui, &mut result.config.exposure_config);
    });

    CollapsingHeader::new("First Layers").show(ui, |ui| {
        exposure_changed |= exposure_config(ui, &mut result.config.first_exposure_config);
    });

    let raster = result.inner.as_raster_mut().unwrap();
    if exposure_changed {
        raster.print_time = result.config.print_time(raster.layers.len() as u32);
        for (i, layer) in raster.layers.iter_mut().enumerate() {
            layer.exposure = result.config.exposure_config(i as u32).into_owned();
        }
    }

    let layer = &mut raster.layers[state.preview_layer - 1];
    layer.unique_exposure = collapsing_toggle(
        "Current Layer Override",
        layer.unique_exposure,
        |ui| {
            ui.add_enabled_ui(layer.unique_exposure, |ui| {
                exposure_config(ui, &mut layer.exposure);
            });
        },
        true,
        ui,
    );

    ui.add_space(8.0);
    ui.heading("Analysis");
    CollapsingHeader::new("Surface Area")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(SURFACE_AREA_DESC);
            ui.add_space(8.0);

            Plot::new("surface_area")
                .width(ui.available_width())
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .allow_boxed_zoom(false)
                .view_aspect(3.0)
                .show(ui, |plot| {
                    let px_area = result.config.pixel_area();
                    let layers = &result.inner.as_raster().unwrap().layers;
                    let series = layers
                        .iter()
                        .enumerate()
                        .map(|(x, layer)| {
                            let area = layer.area as f32 * px_area;
                            [x as f64, area.get::<Centimeter>() as f64]
                        })
                        .collect::<Vec<_>>();
                    plot.add(Line::new("", series).color(Color32::WHITE));
                    plot.add(
                        VLine::new("", (state.preview_layer - 1) as f32)
                            .color(Color32::RED)
                            .style(LineStyle::Dashed { length: 4.0 }),
                    );
                });
        });
}
