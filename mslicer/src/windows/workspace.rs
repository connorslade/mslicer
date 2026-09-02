use const_format::concatcp;
use egui::{CollapsingHeader, Color32, ComboBox, Context, DragValue, Grid, Theme, Ui, Widget};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, ARROWS_CLOCKWISE, FOLDER, INFO};
use egui_plot::{Line, Plot};
use tracing::error;

use crate::{
    app::{
        App,
        config::{
            render::{Projection, RenderStyle},
            ui::UpdateCheckFrequency,
        },
    },
    ui::components::{collapsing_toggle, dragger, grid, vec2_dragger, vec3_dragger},
};

const BASIS_TIP: &str = "Set size to 0px to disable.";
const AA_DESC: &str = "Anti-aliasing smooths jagged edges at the borders of models.";
const SSAO_DESC: &str = "Ambient occlusion (SSAO) simulates how ambient light gets blocked in convex areas, making the rendering a little more realistic.";
const SSAO_SCALE_TIP: &str = "Calculate ambient occlusion at a lower resolution to get better performance at the cost of quality.";
const SPACENAV_CONNECTED: &str = "Connected to Spacenav.";
const SPACENAV_UNCONNECTED: &str =
    "Failed to connect to Spacenav. Make sure the daemon is running and reconnect.";

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.heading("Workspace");

    ui.horizontal_wrapped(|ui| {
        if ui
            .button(concatcp!(FOLDER, " Open Config Directory"))
            .clicked()
            && let Err(err) = open::that_detached(&app.config_dir)
        {
            error!("Failed to open config directory: {}", err);
        }

        if ui
            .button(concatcp!(ARROW_COUNTER_CLOCKWISE, " Reset Config"))
            .clicked()
        {
            app.config = Default::default();
        }
    });
    ui.add_space(8.0);

    Grid::new("theme")
        .spacing([40.0, 4.0])
        .striped(true)
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Theme");
            ComboBox::from_id_salt("theme")
                .selected_text(match app.config.ui.theme {
                    Theme::Dark => "Dark",
                    Theme::Light => "Light",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.config.ui.theme, Theme::Dark, "Dark");
                    ui.selectable_value(&mut app.config.ui.theme, Theme::Light, "Light");
                });
            ui.end_row();

            ui.label("Check for Updates");
            ComboBox::from_id_salt("update_frequency")
                .selected_text(app.config.ui.update_check.name())
                .show_ui(ui, |ui| {
                    for freq in UpdateCheckFrequency::ALL {
                        ui.selectable_value(&mut app.config.ui.update_check, freq, freq.name());
                    }
                });
            ui.end_row();

            ui.horizontal(|ui| {
                ui.label("Render Style");
                ui.label(INFO)
                    .on_hover_text("This setting is really only intended for debugging.");
            });
            ComboBox::from_id_salt("render_style")
                .selected_text(app.config.render.style.name())
                .show_ui(ui, |ui| {
                    for style in RenderStyle::ALL {
                        ui.selectable_value(&mut app.config.render.style, style, style.name());
                    }
                });
            ui.end_row();

            ui.label("Projection");
            ComboBox::from_id_salt("projection")
                .selected_text(app.config.render.projection.name())
                .show_ui(ui, |ui| {
                    for camera in Projection::ALL {
                        ui.selectable_value(
                            &mut app.config.render.projection,
                            camera,
                            camera.name(),
                        );
                    }
                });
            ui.end_row();

            ui.label("Grid Size");
            ui.horizontal(|ui| {
                dragger(ui, "", &mut app.config.render.grid_size, |x| {
                    x.speed(0.1).range(1.0..=f32::MAX)
                });
                ui.take_available_width();
            });
            ui.end_row();

            ui.horizontal(|ui| {
                ui.label("Basis Size");
                ui.label(INFO).on_hover_text(BASIS_TIP);
            });
            DragValue::new(&mut app.config.render.basis_size)
                .range(0.0..=200.0)
                .suffix(" px")
                .ui(ui);
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.checkbox(&mut app.config.render.normals, "Show Normals");
    ui.add_space(8.0);

    ui.heading("Rendering");

    ui.collapsing("Camera", |ui| {
        if ui
            .button(concatcp!(ARROW_COUNTER_CLOCKWISE, " Reset"))
            .clicked()
        {
            app.camera = Default::default();
        }

        Grid::new("workspace_camera")
            .striped(true)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Target");
                vec3_dragger(ui, app.camera.target.as_mut(), |x| x);
                ui.end_row();

                ui.label("Angle");
                vec2_dragger(ui, app.camera.angle.as_mut(), |x| x);
                ui.end_row();

                ui.label("Distance");
                dragger(ui, "", &mut app.camera.distance, |x| x.speed(5));
                ui.end_row();

                ui.label("Fov");
                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut app.camera.fov).speed(0.01));
                    ui.take_available_width();
                });
                ui.end_row();
            });
    });

    let aa = &mut app.config.render.anti_aliasing;
    aa.enabled = collapsing_toggle(
        "Anti Aliasing",
        aa.enabled,
        |ui| {
            ui.label(AA_DESC);
        },
        false,
        ui,
    );

    let ao = &mut app.config.render.ambient_occlusion;
    ao.enabled = collapsing_toggle(
        "Ambient Occlusion",
        ao.enabled,
        |ui| {
            ui.label(SSAO_DESC);
            ui.add_space(4.0);

            grid("ao").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Scale");
                    ui.label(INFO).on_hover_text(SSAO_SCALE_TIP);
                });
                DragValue::new(&mut ao.scale).range(0.1..=1.0).ui(ui);
                ui.end_row();

                ui.label("Samples");
                DragValue::new(&mut ao.samples).ui(ui);
                ui.end_row();

                ui.label("Range");
                DragValue::new(&mut ao.range)
                    .range(0.0..=f32::MAX)
                    .speed(0.1)
                    .ui(ui);
                ui.end_row();

                ui.label("Bias");
                ui.horizontal(|ui| {
                    DragValue::new(&mut ao.bias)
                        .range(0.0..=1.0)
                        .speed(0.001)
                        .ui(ui);
                    ui.take_available_width();
                });
                ui.end_row();
            });

            ui.visuals_mut().collapsing_header_frame = false;
            ui.collapsing("Blur", |ui| {
                grid("ao_blur").show(ui, |ui| {
                    ui.label("Radius");
                    DragValue::new(&mut ao.blur_radius).range(0..=16).ui(ui);
                    ui.end_row();

                    ui.label("Spatial");
                    DragValue::new(&mut ao.blur_spatial)
                        .range(0.01..=f32::MAX)
                        .ui(ui);
                    ui.end_row();

                    ui.label("Depth");
                    DragValue::new(&mut ao.blur_depth)
                        .range(0.01..=f32::MAX)
                        .ui(ui);
                    ui.end_row();

                    ui.label("Normal");
                    ui.horizontal(|ui| {
                        DragValue::new(&mut ao.blur_normal)
                            .range(0.01..=f32::MAX)
                            .ui(ui);
                        ui.take_available_width();
                    });
                    ui.end_row();
                });
            });
        },
        false,
        ui,
    );

    ui.add_space(8.0);
    ui.heading("Miscellaneous");
    CollapsingHeader::new("Spacenav")
        .enabled(cfg!(unix))
        .show(ui, |ui| {
            if app.spacenav.is_connected() {
                ui.label(SPACENAV_CONNECTED);
            } else {
                ui.label(SPACENAV_UNCONNECTED);
                ui.add_space(8.0);
                ui.button(concatcp!(ARROWS_CLOCKWISE, " Reconnect"))
                    .clicked()
                    .then(|| app.spacenav.try_connect());
            }

            ui.add_space(8.0);
            Grid::new("spacenav")
                .striped(true)
                .num_columns(2)
                .show(ui, |ui| {
                    let dragger = |val: &mut f32, ui: &mut Ui| {
                        DragValue::new(val)
                            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                            .ui(ui)
                    };

                    let config = &mut app.config.spacenav;

                    ui.label("Overall Sensitivity");
                    dragger(&mut config.gain, ui);
                    ui.end_row();

                    ui.label("Rotation Sensitivity");
                    dragger(&mut config.rotation_gain, ui);
                    ui.end_row();

                    ui.label("Position Sensitivity");
                    ui.horizontal(|ui| {
                        dragger(&mut config.position_gain, ui);
                        ui.take_available_width();
                    });
                    ui.end_row();
                });
        })
        .header_response
        .on_hover_text("Only supported on Linux systems at the moment.");

    ui.collapsing("Stats", |ui| {
        ui.label(format!(
            "Frame Time: {:.2}ms",
            app.fps.frame_time() * 1000.0
        ));
        ui.label(format!("FPS: {:.2}", 1.0 / app.fps.frame_time()));
        ui.add_space(4.0);

        Plot::new("fps")
            .width(ui.available_width())
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .show_axes([false, true])
            .view_aspect(3.0)
            .show(ui, |plot| {
                let series = app
                    .fps
                    .fps_history()
                    .enumerate()
                    .map(|(x, y)| [x as f64, y as f64])
                    .collect::<Vec<_>>();
                plot.add(Line::new("", series).color(Color32::WHITE));
            });
        ui.add_space(4.0);
    });
}
