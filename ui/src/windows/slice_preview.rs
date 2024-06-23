use eframe::Frame;
use egui::{ColorImage, Context, DragValue, Image, Slider, TextureOptions, Window};
use goo_format::LayerDecoder;

use crate::app::App;

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    if let Some(slice_progress) = &app.slice_progress {
        if let Some(result) = slice_progress.result.lock().unwrap().as_mut() {
            Window::new("Slice Preview")
                .resizable([true, true])
                .show(ctx, move |ui| {
                    let last_layer = result.slice_preview_layer;

                    ui.horizontal(|ui| {
                        ui.add(
                            Slider::new(
                                &mut result.slice_preview_layer,
                                0..=result.goo.layers.len(),
                            )
                            .vertical()
                            .show_value(false),
                        );

                        if result.current_preview.is_none() {
                            let (width, height) = (
                                result.goo.header.x_resolution as u32,
                                result.goo.header.y_resolution as u32,
                            );

                            let layer_data = &result.goo.layers[result.slice_preview_layer].data;
                            let mut decoder = LayerDecoder::new(layer_data);

                            let mut image = vec![0; (width * height / 4) as usize];
                            let mut pixel = 0;
                            while let Some(run) = decoder.next() {
                                for _ in 0..run.length {
                                    image[pixel / 4] = run.value;
                                    pixel += 1;
                                }
                            }

                            let preview = ctx.load_texture(
                                "slice_preview",
                                ColorImage::from_gray(
                                    [width as usize / 2, height as usize / 2],
                                    &image,
                                ),
                                TextureOptions::default(),
                            );
                            result.current_preview = Some(preview);
                        }

                        ui.add(
                            Image::new(result.current_preview.as_ref().unwrap())
                                .maintain_aspect_ratio(true)
                                .shrink_to_fit(),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.add(
                            DragValue::new(&mut result.slice_preview_layer)
                                .clamp_range(0..=result.goo.layers.len())
                                .suffix(format!("/{}", result.goo.layers.len())),
                        );
                        result.slice_preview_layer += ui.button("+").clicked() as usize;
                        result.slice_preview_layer -= ui.button("-").clicked() as usize;
                    });

                    if last_layer != result.slice_preview_layer {
                        result.current_preview = None;
                    }
                });
        }
    }
}
