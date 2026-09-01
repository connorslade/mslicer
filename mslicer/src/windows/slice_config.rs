use std::borrow::Cow;

use const_format::concatcp;
use egui::{Color32, ComboBox, Context, DragValue, Grid, Ui, Widget, emath::OrderedFloat};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular::{
    ARROW_COUNTER_CLOCKWISE, ARROW_LINE_DOWN, ARROW_LINE_UP, INFO, NOTE_PENCIL, PENCIL, PLUS,
    TIMER, TRASH, WARNING,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use itertools::Itertools;
use nalgebra::Vector2;
use num_integer::cbrt;
use slicer::post_process::{
    elephant_foot_fixer::ElephantFootFixer, variable_layer_height::VariableLayerHeight,
};

use crate::{
    app::{
        App,
        config::{Config, printers::DEFAULT_PRINTERS},
        history::SliceConfigAction,
    },
    ui::{
        components::{
            BeingEditedExt, being_edited, collapsing_toggle, grid, history_tracked_value,
            vec2_dragger,
        },
        popup::{Popup, PopupApp},
        state::{SelectedPrinter, UiState},
    },
};
use common::{
    slice::{ExposureConfig, ExposureRemap, SliceMode},
    units::{Milimeter, Minute, Mircometer},
};

const ANTI_ALIAS_TOOLTIP: &str = "Uses supersampling anti-aliasing (SSAA) to pick grayscale values that more accurately represent the actual model geometry. The actual value of this setting is the number of effective samples per voxel.";
const TRANSITION_LAYER_TOOLTIP: &str = "Transition layers interpolate between the first exposure settings and the normal exposure settings.";

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
    grid("slice_config").show(ui, |ui| {
        ui.label("Slice Mode");
        let old_mode = slice_config.mode;
        let format = slice_config.mode;
        ComboBox::from_id_salt("slice_mode")
            .selected_text(format.name())
            .show_ui(ui, |ui| {
                for format in SliceMode::ALL {
                    ui.selectable_value(&mut slice_config.mode, format, format.name());
                }
            });
        if slice_config.mode != old_mode {
            app.history.track(SliceConfigAction::Mode(old_mode).into());
        }
        ui.end_row();

        ui.label("Printer");
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing.x = 4.0;
            ComboBox::from_id_salt("printer")
                .selected_text(match app.state.selected_printer {
                    SelectedPrinter::Project => Cow::Borrowed("Custom"),
                    SelectedPrinter::Custom(i) => app.config.printers[i].name.clone(),
                    SelectedPrinter::Preset(brand, model) => {
                        let brand = DEFAULT_PRINTERS[brand];
                        Cow::Owned(format!("{} {}", brand.0, brand.1[model].name))
                    }
                })
                .show_ui(ui, |ui| {
                    printer_presets(ui, &mut app.config, &mut app.state);
                });

            if ui.button(PENCIL).clicked() {
                app.popup
                    .open(Popup::new("Edit Your Printer Presets", edit_presets).close_button(true));
            }
        });
        ui.end_row();

        let platform = &mut slice_config.platform_size;
        let prev = *platform;

        match app.state.selected_printer {
            SelectedPrinter::Project => {
                ui.label("Platform Resolution");
                let old_resolution = slice_config.platform_resolution;
                let editing = vec2_dragger(ui, slice_config.platform_resolution.as_mut(), |x| x);
                history_tracked_value(
                    (editing, ui, &mut app.history),
                    ("platform_resolution", || {
                        SliceConfigAction::PlatformResolution(old_resolution).into()
                    }),
                );
                ui.end_row();

                ui.label("Platform Size (mm)");
                let mut editing = false;
                ui.horizontal(|ui| {
                    (DragValue::new(platform.x.raw_mut()).ui(ui)).being_edited(&mut editing);
                    ui.label("×");
                    (DragValue::new(platform.y.raw_mut()).ui(ui)).being_edited(&mut editing);
                    ui.label("×");
                    (DragValue::new(platform.z.raw_mut()).ui(ui)).being_edited(&mut editing);
                });
                history_tracked_value(
                    (editing, ui, &mut app.history),
                    ("platform_size", || {
                        SliceConfigAction::PlatformSize(prev).into()
                    }),
                );
                ui.end_row();
            }
            SelectedPrinter::Custom(idx) => {
                let printer = &app.config.printers[idx];
                let (old_resolution, old_size) = (slice_config.platform_resolution, *platform);
                slice_config.platform_resolution = printer.resolution;
                *platform = printer.size;
                if slice_config.platform_resolution != old_resolution || *platform != old_size {
                    app.history
                        .track(SliceConfigAction::PlatformResolution(old_resolution).into());
                    app.history
                        .track(SliceConfigAction::PlatformSize(old_size).into());
                }
            }
            SelectedPrinter::Preset(brand, model) => {
                let printer = &DEFAULT_PRINTERS[brand].1[model];
                let (old_resolution, old_size) = (slice_config.platform_resolution, *platform);
                slice_config.platform_resolution = printer.resolution;
                *platform = printer.size;

                if slice_config.platform_resolution != old_resolution {
                    app.history
                        .track(SliceConfigAction::PlatformResolution(old_resolution).into());
                }
                if *platform != old_size {
                    app.history
                        .track(SliceConfigAction::PlatformSize(old_size).into());
                }
            }
        }

        if *platform != prev {
            (app.project.models.iter_mut())
                .for_each(|model| model.update_oob(&slice_config.platform_size));
        }

        ui.label("Slice Height");
        ui.horizontal(|ui| {
            let old_slice_height = slice_config.slice_height;
            let mut editing = false;
            slice_config.slice_height.with::<Mircometer>(|value| {
                let response = DragValue::new(value)
                    .suffix(" μm")
                    .range(1.0..=f32::MAX)
                    .ui(ui);
                editing = being_edited(&response);
            });
            history_tracked_value(
                (editing, ui, &mut app.history),
                ("slice_height", || {
                    SliceConfigAction::SliceHeight(old_slice_height).into()
                }),
            );
            ui.take_available_width();
        });
        ui.end_row();

        ui.horizontal(|ui| {
            ui.label("Anti Aliasing");
            ui.label(INFO).on_hover_text(ANTI_ALIAS_TOOLTIP);
        });
        ui.horizontal(|ui| {
            let old_supersample = slice_config.supersample;
            let response = DragValue::new(&mut slice_config.supersample)
                .custom_formatter(|val, _| (val as u32).pow(3).to_string())
                .custom_parser(|val| val.parse::<u32>().ok().map(|x| cbrt(x) as f64))
                .suffix("×")
                .speed(0.1)
                .range(1..=16)
                .ui(ui);
            history_tracked_value(
                (being_edited(&response), ui, &mut app.history),
                ("supersample", || {
                    SliceConfigAction::Supersample(old_supersample).into()
                }),
            );
        });
        ui.end_row();

        ui.label("First Layers");
        let old_first_layers = slice_config.first_layers;
        let response = DragValue::new(&mut slice_config.first_layers).ui(ui);
        history_tracked_value(
            (being_edited(&response), ui, &mut app.history),
            ("first_layers", || {
                SliceConfigAction::FirstLayers(old_first_layers).into()
            }),
        );
        ui.end_row();

        ui.horizontal(|ui| {
            ui.label("Transition Layers");
            ui.label(INFO).on_hover_text(TRANSITION_LAYER_TOOLTIP);
        });
        let old_transition_layers = slice_config.transition_layers;
        let response = DragValue::new(&mut slice_config.transition_layers).ui(ui);
        history_tracked_value(
            (being_edited(&response), ui, &mut app.history),
            ("transition_layers", || {
                SliceConfigAction::TransitionLayers(old_transition_layers).into()
            }),
        );
        ui.end_row();
    });

    ui.add_space(8.0);
    ui.collapsing("Normal Layers", |ui| {
        let editing = exposure_config(ui, &mut slice_config.exposure_config);
        history_tracked_value(
            (editing, ui, &mut app.history),
            ("normal_exposure", || {
                SliceConfigAction::NormalExposure(slice_config.exposure_config.clone()).into()
            }),
        );
    });

    ui.collapsing("First Layers", |ui| {
        let editing = exposure_config(ui, &mut slice_config.first_exposure_config);
        history_tracked_value(
            (editing, ui, &mut app.history),
            ("first_exposure", || {
                SliceConfigAction::FirstExposure(slice_config.first_exposure_config.clone()).into()
            }),
        );
    });

    ui.add_space(8.0);
    ui.heading("Post Processing");

    ui.collapsing("Exposure Remapping", |ui| {
        exposure_remapping(
            &mut slice_config.exposure_remap,
            &mut app.state.selected_remap_point,
            ui,
        );
        let editing = app.state.selected_remap_point.is_some();
        history_tracked_value(
            (editing, ui, &mut app.history),
            ("exposure_remap", || {
                SliceConfigAction::ExposureRemap(slice_config.exposure_remap.clone()).into()
            }),
        );
        ui.add_space(8.0);
    });

    let post_processing = &mut app.project.post_processing;

    let old_variable_layer_height = post_processing.variable_layer_height.clone();
    let mut editing = false;
    post_processing.variable_layer_height.enabled = collapsing_toggle(
        "Variable Layer Height",
        old_variable_layer_height.enabled,
        |ui| editing = variable_layer_height(&mut post_processing.variable_layer_height, ui),
        false,
        ui,
    );
    if post_processing.variable_layer_height.enabled != old_variable_layer_height.enabled {
        app.history
            .track(SliceConfigAction::VariableLayerHeight(old_variable_layer_height).into());
    }
    history_tracked_value(
        (editing, ui, &mut app.history),
        ("variable_layer_height", || {
            SliceConfigAction::VariableLayerHeight(post_processing.variable_layer_height.clone())
                .into()
        }),
    );

    let old_elephant_foot_fixer = post_processing.elephant_foot_fixer.clone();
    let mut editing = false;
    post_processing.elephant_foot_fixer.enabled = collapsing_toggle(
        "Elephant Foot Fixer",
        old_elephant_foot_fixer.enabled,
        |ui| editing = elephant_foot_fixer(&mut post_processing.elephant_foot_fixer, ui),
        false,
        ui,
    );
    if post_processing.elephant_foot_fixer.enabled != old_elephant_foot_fixer.enabled {
        app.history
            .track(SliceConfigAction::ElephantFootFixer(old_elephant_foot_fixer).into());
    }
    history_tracked_value(
        (editing, ui, &mut app.history),
        ("elephant_foot_fixer", || {
            SliceConfigAction::ElephantFootFixer(post_processing.elephant_foot_fixer.clone()).into()
        }),
    );
}

fn printer_presets(ui: &mut Ui, config: &mut Config, state: &mut UiState) {
    let this_selected = matches!(state.selected_printer, SelectedPrinter::Project);
    if ui.selectable_label(this_selected, "Custom").clicked() {
        state.selected_printer = SelectedPrinter::Project;
    }

    ui.menu_button("Your Presets", |ui| {
        for (i, printer) in config.printers.iter().enumerate() {
            let res = printer.resolution;
            let size = printer.size.map(|x| x.get::<Milimeter>());

            let this_selected =
                matches!(state.selected_printer, SelectedPrinter::Custom(x) if x == i);
            if ui
                .selectable_label(this_selected, &*printer.name)
                .on_hover_text(format!(
                    "{}x{} ({}x{}x{})",
                    res.x, res.y, size.x, size.y, size.z
                ))
                .clicked()
            {
                state.selected_printer = SelectedPrinter::Custom(i);
            }
        }
    });

    ui.separator();
    for (i, (brand, models)) in DEFAULT_PRINTERS.iter().enumerate() {
        ui.menu_button(*brand, |ui| {
            for (j, model) in models.iter().enumerate() {
                let res = model.resolution;
                let size = model.size.map(|x| x.get::<Milimeter>());

                let this_selected = matches!(state.selected_printer,
                    SelectedPrinter::Preset(brand, model) if brand == i && model == j);
                if ui
                    .selectable_label(this_selected, &*model.name)
                    .on_hover_text(format!(
                        "{}x{} ({}x{}x{})",
                        res.x, res.y, size.x, size.y, size.z
                    ))
                    .clicked()
                {
                    state.selected_printer = SelectedPrinter::Preset(i, j);
                }
            }
        });
    }
}

// todo: visually merge lift/retract settings (ARROW_LINE_UP / DOWN)
pub fn exposure_config(ui: &mut Ui, config: &mut ExposureConfig) -> bool {
    let mut editing = false;
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(80.0))
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::remainder())
        .header(16.0, |mut row| {
            row.col(|ui| {
                ui.label("Exposure");
            });

            row.col(|ui| {
                DragValue::new(config.exposure_time.raw_mut())
                    .suffix(" s")
                    .speed(0.1)
                    .range(0.0..=f32::MAX)
                    .ui(ui)
                    .being_edited(&mut editing);
            });

            row.col(|ui| {
                ui.label("@");
            });

            row.col(|ui| {
                ui.horizontal(|ui| {
                    let mut pwm = config.pwm as f32 / 2.55;
                    DragValue::new(&mut pwm)
                        .max_decimals(0)
                        .suffix('%')
                        .ui(ui)
                        .being_edited(&mut editing);
                    config.pwm = (pwm * 2.55).round() as u8;

                    ui.label(TIMER).on_hover_text("Exposure delay");
                    DragValue::new(config.exposure_delay.raw_mut())
                        .suffix(" s")
                        .speed(0.1)
                        .range(0.0..=f32::MAX)
                        .ui(ui)
                        .being_edited(&mut editing);
                });
            });
        })
        .body(|mut body| {
            body.row(16.0, |mut row| {
                row.col(|ui| {
                    ui.label("Lift");
                });

                row.col(|ui| {
                    DragValue::new(config.lift_distance.raw_mut())
                        .suffix(" mm")
                        .speed(0.1)
                        .range(0.0..=f32::MAX)
                        .ui(ui)
                        .being_edited(&mut editing);
                });

                row.col(|ui| {
                    ui.label("@");
                });

                // todo: needs refinement
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        config.lift_speed.with::<Milimeter, Minute>(|val| {
                            DragValue::new(val)
                                .suffix(" mm/min")
                                .speed(0.1)
                                .range(0.0..=f32::MAX)
                                .ui(ui)
                                .being_edited(&mut editing);
                        });
                        ui.label(ARROW_LINE_UP).on_hover_text("Lift speed");
                        ui.add_space(4.0);
                    });

                    ui.horizontal(|ui| {
                        config.retract_speed.with::<Milimeter, Minute>(|val| {
                            DragValue::new(val)
                                .suffix(" mm/min")
                                .speed(0.1)
                                .range(0.0..=f32::MAX)
                                .ui(ui)
                                .being_edited(&mut editing);
                        });
                        ui.label(ARROW_LINE_DOWN).on_hover_text("Retract speed");
                    });
                });
            });
        });

    editing
}

fn edit_presets(app: &mut PopupApp, ui: &mut Ui) -> bool {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::auto())
        .column(Column::exact(150.0))
        .column(Column::auto())
        .column(Column::auto())
        .header(16.0, |mut row| {
            row.col(|_ui| {});
            for label in ["Name", "Resolution", "Size"] {
                row.col(|ui| {
                    ui.label(label);
                });
            }
        })
        .body(|mut body| {
            let mut delete = None;
            for (i, preset) in app.config.printers.iter_mut().enumerate() {
                body.row(16.0, |mut row| {
                    row.col(|ui| {
                        ui.visuals_mut().button_frame = false;
                        if ui.button(TRASH).clicked() {
                            delete = Some(i);
                        }
                    });

                    row.col(|ui| {
                        ui.text_edit_singleline(&mut preset.name);
                    });

                    row.col(|ui| {
                        vec2_dragger(ui, preset.resolution.as_mut(), |x| x);
                    });

                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(DragValue::new(preset.size.x.raw_mut()).fixed_decimals(2));
                            ui.label("×");
                            ui.add(DragValue::new(preset.size.y.raw_mut()).fixed_decimals(2));
                            ui.label("×");
                            ui.add(DragValue::new(preset.size.z.raw_mut()).fixed_decimals(2));
                        });
                    });
                });
            }

            if let Some(delete) = delete {
                app.config.printers.remove(delete);
            }
        });

    ui.add_space(8.0);
    ui.vertical_centered(|ui| {
        if ui.button(concatcp!(PLUS, " New")).clicked() {
            app.config.printers.push(Default::default());
        }
    });

    false
}

fn exposure_remapping(
    remap: &mut ExposureRemap,
    selected_remap_point: &mut Option<u8>,
    ui: &mut Ui,
) {
    const EXPOSURE_REMAPPING_DESCRIPTION: &str = "Fractional exposure values don't necessarily correspond linearly to voxel growth, so you may need to adjust the mapping with the curve below to get ideal results with antialiasing.";
    ui.label(EXPOSURE_REMAPPING_DESCRIPTION);
    let [p1, p2, p3, p4] = remap.points();

    let mut pointer = None;
    let plot = Plot::new("exposure_remap")
        .width(ui.available_width() / 2.0)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .view_aspect(1.0)
        .show_axes([false; 2])
        .default_x_bounds(-0.1, 1.1)
        .default_y_bounds(-0.1, 1.1)
        .auto_bounds([false; 2])
        .show(ui, |plot| {
            let mut points = Vec::new();
            for i in 0..100 {
                let p = remap.bezier(i as f32 / 99.0).map(|x| x as f64);
                points.push([p.x, p.y]);
            }

            pointer = (plot.pointer_coordinate())
                .map(|x| Vector2::new(x.x, x.y).map(|x| x.clamp(0.0, 1.0)));
            if let Some(selected_remap_point) = selected_remap_point
                && let Some(pointer) = pointer
            {
                let pointer = Vector2::new(pointer.x, pointer.y).map(|x| x as f32);
                match *selected_remap_point {
                    0 => remap.start = pointer.y,
                    1 => remap.control[0] = pointer - Vector2::new(0.0, remap.start),
                    2 => remap.control[1] = pointer - Vector2::new(1.0, remap.end),
                    3 => remap.end = pointer.y,
                    _ => unreachable!(),
                }
            }

            let [p1, p2, p3, p4] = [p1, p2, p3, p4].map(|x| [x.x, x.y].map(|x| x as f64));
            plot.line(Line::new("", points));
            plot.line(Line::new("", vec![p1, p2]).color(Color32::GRAY));
            plot.line(Line::new("", vec![p3, p4]).color(Color32::GRAY));
            plot.points(
                Points::new("", vec![p1, p2, p3, p4])
                    .shape(MarkerShape::Circle)
                    .radius(5.0_f32)
                    .color(Color32::GRAY),
            );
        });

    if let Some(pointer) = pointer
        && ui.input(|x| x.pointer.button_pressed(egui::PointerButton::Primary))
        && plot.response.contains_pointer()
    {
        let pointer = Vector2::new(pointer.x, pointer.y).cast::<f32>();
        *selected_remap_point = ([p1, p2, p3, p4].iter())
            .position_min_by_key(|x| OrderedFloat((*x - pointer).magnitude()))
            .map(|x| x as u8);
    }

    if ui.input(|x| x.pointer.button_released(egui::PointerButton::Primary)) {
        *selected_remap_point = None;
    }
}

fn variable_layer_height(this: &mut VariableLayerHeight, ui: &mut Ui) -> bool {
    const VARIABLE_LAYER_HEIGHT_THRESHOLD_TOOLTIP: &str =
        "Maximum allowed average value deviation between layers (in value/mm²).";

    ui.label("Merges identical layers into thicker layers to reduce printing time while retaining resolution where needed.");
    ui.label("Note that with this post processor enabled you won't be able to save as NanoDLP.");
    ui.add_space(8.0);

    let mut editing = false;
    grid("variable_layer_height").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Threshold");
            ui.label(INFO)
                .on_hover_text(VARIABLE_LAYER_HEIGHT_THRESHOLD_TOOLTIP);
        });
        DragValue::new(&mut this.threshold)
            .ui(ui)
            .being_edited(&mut editing);
        ui.end_row();

        ui.label("Max Layers");
        DragValue::new(&mut this.max_layers)
            .ui(ui)
            .being_edited(&mut editing);
        ui.end_row();

        ui.label("Exposure");
        DragValue::new(this.exposure.raw_mut())
            .suffix(" s")
            .ui(ui)
            .being_edited(&mut editing);
        ui.end_row();
    });

    editing
}

fn elephant_foot_fixer(this: &mut ElephantFootFixer, ui: &mut Ui) -> bool {
    ui.label("Fixes the 'Elephant Foot' effect by exposing the edges of the bottom layers at a lower intensity. You may have to make a few test prints to find the right settings for your printer and resin.");
    ui.add_space(8.0);

    const INSET_TIP: &str = "The distance in from the edges that will have a reduced intensity.";
    const INSET_WARNING: &str = "Larger values will drastically increase post-processing time.";
    const INTENSITY_TIP: &str =
        "This percent will be multiplied by the pixel values of the edge pixels.";

    let mut editing = false;
    Grid::new("elephant_foot_fixer")
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
                DragValue::new(&mut this.inset_distance)
                    .speed(0.1)
                    .range(0.1..=f32::MAX)
                    .ui(ui)
                    .being_edited(&mut editing);
                ui.take_available_width();
            });
            ui.end_row();

            ui.horizontal(|ui| {
                ui.label("Intensity");
                ui.label(INFO).on_hover_text(INTENSITY_TIP);
            });
            ui.horizontal(|ui| {
                DragValue::new(&mut this.intensity_multiplier)
                    .range(0.0..=100.0)
                    .speed(1)
                    .suffix("%")
                    .ui(ui)
                    .being_edited(&mut editing);
            });
        });

    editing
}
