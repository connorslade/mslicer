use egui::{Align, ComboBox, Context, DragValue, Grid, Layout, Ui};
use egui_phosphor::regular::{INFO, WARNING};
use slicer::post_process::{anti_alias::AntiAlias, elephant_foot_fixer::ElephantFootFixer};

use crate::{
    app::App,
    ui::components::{dragger, dragger_tip, vec2_dragger, vec3_dragger},
};
use common::{config::ExposureConfig, format::Format};

const TRANSITION_LAYER_TOOLTIP: &str = "Transition layers interpolate between the first exposure settings and the normal exposure settings.";
const SLICE_FORMAT_TOOLTIP: &str =
    "Only .goo and .ctb files can be sent with the 'Remote Print' module.";

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let slice_config = &mut app.project.slice_config;
    Grid::new("slice_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Slice Format");
                ui.label(INFO).on_hover_text(SLICE_FORMAT_TOOLTIP);
            });
            let format = slice_config.format;
            ComboBox::new("slice_format", "")
                .selected_text(format!("{} (.{})", format.name(), format.extension()))
                .show_ui(ui, |ui| {
                    for format in Format::ALL {
                        ui.selectable_value(&mut slice_config.format, format, format.name());
                    }
                });
            ui.end_row();

            ui.label("Printer");
            ComboBox::new("printer", "")
                .selected_text(match app.state.selected_printer {
                    0 => "Custom",
                    i => &app.config.printers[i - 1].name,
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.state.selected_printer, 0, "Custom");
                    for (i, printer) in app.config.printers.iter().enumerate() {
                        let (res, size) = (printer.resolution, printer.size);
                        ui.selectable_value(&mut app.state.selected_printer, i + 1, &printer.name)
                            .on_hover_text(format!(
                                "{}x{} ({}x{}x{})",
                                res.x, res.y, size.x, size.y, size.z
                            ));
                    }
                });
            ui.end_row();

            let platform = &mut slice_config.platform_size;
            let prev = *platform;
            if app.state.selected_printer == 0 {
                ui.label("Platform Resolution");
                vec2_dragger(ui, slice_config.platform_resolution.as_mut(), |x| x);
                ui.end_row();

                ui.label("Platform Size (mm)");
                vec3_dragger(ui, platform.as_mut(), |x| x);
                ui.end_row();
            } else {
                let printer = &app.config.printers[app.state.selected_printer - 1];
                slice_config.platform_resolution = printer.resolution;
                *platform = printer.size;
            }

            if *platform != prev {
                (app.project.models.iter_mut())
                    .for_each(|model| model.update_oob(&slice_config.platform_size));
            }

            ui.label("Slice Height (mm)");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut slice_config.slice_height));
                ui.add_space(ui.available_width());
            });
            ui.end_row();

            ui.label("First Layers");
            ui.add(DragValue::new(&mut slice_config.first_layers));
            ui.end_row();

            ui.horizontal(|ui| {
                ui.label("Transition Layers");
                ui.label(INFO).on_hover_text(TRANSITION_LAYER_TOOLTIP);
            });
            ui.add(DragValue::new(&mut slice_config.transition_layers));
            ui.end_row();
        });

    ui.collapsing("Exposure Config", |ui| {
        exposure_config_grid(ui, &mut slice_config.exposure_config);
    });

    ui.collapsing("First Exposure Config", |ui| {
        exposure_config_grid(ui, &mut slice_config.first_exposure_config);
    });

    ui.add_space(16.0);
    ui.heading("Post Processing");

    let post_processing = &mut app.project.post_processing;
    ui.collapsing("Anti Alias", |ui| {
        anti_alias(&mut post_processing.anti_alias, ui)
    });
    ui.collapsing("Elephant Foot Fixer", |ui| {
        elephant_foot_fixer(&mut post_processing.elephant_foot_fixer, ui)
    });
}

fn exposure_config_grid(ui: &mut Ui, config: &mut ExposureConfig) {
    Grid::new("exposure_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Exposure Time (s)");
            ui.add(DragValue::new(&mut config.exposure_time).range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Lift Distance (mm)");
            ui.add(DragValue::new(&mut config.lift_distance).range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Lift Speed (cm/min)");
            ui.add(DragValue::new(&mut config.lift_speed).range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Retract Distance (mm)");
            ui.add(DragValue::new(&mut config.retract_distance).range(0.0..=f32::MAX));
            ui.end_row();

            ui.label("Retract Speed (cm/min)");
            ui.add(DragValue::new(&mut config.retract_speed).range(0.0..=f32::MAX));
            ui.end_row();
        });
}

pub fn anti_alias(this: &mut AntiAlias, ui: &mut Ui) {
    ui.label("Applies a blur to each layer to smooth the edges.");
    ui.checkbox(&mut this.enabled, "Enabled");

    ui.add_space(8.0);
    dragger(ui, "Radius", &mut this.radius, |x| {
        x.speed(0.1).range(0.1..=10.0)
    });
}

pub fn elephant_foot_fixer(this: &mut ElephantFootFixer, ui: &mut Ui) {
    ui.label("Fixes the 'Elephant Foot' effect by exposing the edges of the bottom layers at a lower intensity. You may have to make a few test prints to find the right settings for your printer and resin.");
    ui.checkbox(&mut this.enabled, "Enabled");

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        dragger(ui, "Inset Distance", &mut this.inset_distance, |x| {
            x.speed(0.1).suffix("mm")
        });
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.label(INFO).on_hover_text(
                "The distance in from the edges that will have a reduced intensity.",
            );
            ui.label(WARNING)
                .on_hover_text("Larger values will drastically increase post-processing time.");
            ui.add_space(ui.available_width());
        })
    });

    dragger_tip(
        ui,
        "Intensity",
        "This percent will be multiplied by the pixel values of the edge pixels.",
        &mut this.intensity_multiplier,
        |x| x.range(0.0..=100.0).speed(1).suffix("%"),
    );
}
