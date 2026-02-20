use const_format::concatcp;
use egui::{ComboBox, Context, DragValue, Grid, Ui};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, INFO, NOTE_PENCIL, WARNING};
use slicer::post_process::{anti_alias::AntiAlias, elephant_foot_fixer::ElephantFootFixer};

use crate::{
    app::{App, history::ConfigAction},
    ui::components::{dragger, history_tracked_value, metric_dragger, vec2_dragger},
    util::WidgetExt,
};
use common::{
    slice::{ExposureConfig, Format},
    units::Milimeter,
};

const TRANSITION_LAYER_TOOLTIP: &str = "Transition layers interpolate between the first exposure settings and the normal exposure settings.";
const SLICE_FORMAT_TOOLTIP: &str =
    "Only .goo and .ctb files can be sent with the 'Remote Print' module.";

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.heading("Slice Config");

    ui.add_enabled_ui(
        app.project.slice_config != app.config.default_slice_config,
        |ui| {
            ui.horizontal(|ui| {
                ui.button(concatcp!(ARROW_COUNTER_CLOCKWISE, " Reset to Default"))
                    .clicked()
                    .then(|| app.project.slice_config = app.config.default_slice_config.clone());
                (ui.button(concatcp!(NOTE_PENCIL, " Set Default")).clicked())
                    .then(|| app.config.default_slice_config = app.project.slice_config.clone());
            });
        },
    );
    ui.add_space(8.0);

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
                    let printer = app.state.selected_printer;
                    ui.selectable_value(&mut app.state.selected_printer, 0, "Custom");
                    for (i, printer) in app.config.printers.iter().enumerate() {
                        let res = printer.resolution;
                        let size = printer.size.map(|x| x.get::<Milimeter>());

                        ui.selectable_value(&mut app.state.selected_printer, i + 1, &printer.name)
                            .on_hover_text(format!(
                                "{}x{} ({}x{}x{})",
                                res.x, res.y, size.x, size.y, size.z
                            ));
                    }

                    (app.state.selected_printer != printer).then(|| {
                        app.history
                            .track_config(ConfigAction::SelectedPrinter(printer))
                    });
                });
            ui.end_row();

            let platform = &mut slice_config.platform_size;
            let prev = *platform;
            if app.state.selected_printer == 0 {
                ui.label("Platform Resolution");
                let original_resolution = slice_config.platform_resolution;
                let editing = vec2_dragger(ui, slice_config.platform_resolution.as_mut(), |x| x);
                history_tracked_value(
                    (editing, ui, &mut app.history),
                    (0, || {
                        ConfigAction::PrinterResolution(original_resolution).into()
                    }),
                );
                ui.end_row();

                ui.label("Platform Size");
                ui.horizontal(|ui| {
                    ui.add(DragValue::new(platform.x.raw_mut()));
                    ui.label("×");
                    ui.add(DragValue::new(platform.y.raw_mut()));
                    ui.label("×");
                    ui.add(DragValue::new(platform.z.raw_mut()));
                });
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

            ui.label("Slice Height");
            ui.horizontal(|ui| {
                ui.add(metric_dragger(
                    slice_config.slice_height.raw_mut(),
                    ("m", -3, false),
                ));
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
        exposure_config(ui, &mut slice_config.exposure_config);
    });

    ui.collapsing("First Exposure Config", |ui| {
        exposure_config(ui, &mut slice_config.first_exposure_config);
    });

    ui.add_space(16.0);
    ui.heading("Post Processing");

    ui.label(
        "These effects are currently not optimized and will significancy increase slicing time.",
    );

    let post_processing = &mut app.project.post_processing;
    ui.collapsing("Anti Alias", |ui| {
        anti_alias(&mut post_processing.anti_alias, ui)
    });
    ui.collapsing("Elephant Foot Fixer", |ui| {
        elephant_foot_fixer(&mut post_processing.elephant_foot_fixer, ui)
    });
}

fn exposure_config(ui: &mut Ui, config: &mut ExposureConfig) {
    Grid::new("exposure_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Exposure Time");
            ui.horizontal(|ui| {
                metric_dragger(config.exposure_time.raw_mut(), ("s", 0, false))
                    .range(0.0..=f32::MAX)
                    .add(ui);
                ui.add_space(ui.available_width());
            });
            ui.end_row();

            ui.label("Lift Distance ");
            metric_dragger(config.lift_distance.raw_mut(), ("m", -3, false))
                .range(0.0..=f32::MAX)
                .add(ui);
            ui.end_row();

            ui.label("Lift Speed");
            metric_dragger(config.lift_speed.raw_mut(), ("m/s", -2, true))
                .range(0.0..=f32::MAX)
                .add(ui);
            ui.end_row();

            ui.label("Retract Distance");
            metric_dragger(config.retract_distance.raw_mut(), ("m", -3, false))
                .range(0.0..=f32::MAX)
                .add(ui);
            ui.end_row();

            ui.label("Retract Speed");
            metric_dragger(config.retract_speed.raw_mut(), ("m/s", -2, true))
                .range(0.0..=f32::MAX)
                .add(ui);
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

    const INSET_TIP: &str = "The distance in from the edges that will have a reduced intensity.";
    const INSET_WARNING: &str = "Larger values will drastically increase post-processing time.";
    const INTENSITY_TIP: &str =
        "This percent will be multiplied by the pixel values of the edge pixels.";

    Grid::new("exposure_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Inset Distance");
                ui.label(INFO).on_hover_text(INSET_TIP);
                ui.label(WARNING).on_hover_text(INSET_WARNING);
            });
            ui.horizontal(|ui| {
                ui.add(metric_dragger(&mut this.inset_distance, ("m", -3, false)));
                ui.add_space(ui.available_width());
            });
            ui.end_row();

            ui.horizontal(|ui| {
                ui.label("Intensity");
                ui.label(INFO).on_hover_text(INTENSITY_TIP);
            });
            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut this.intensity_multiplier)
                        .range(0.0..=100.0)
                        .speed(1)
                        .suffix("%"),
                );
            });
        });
}
