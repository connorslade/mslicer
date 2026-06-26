use const_format::concatcp;
use egui::{CollapsingHeader, ComboBox, Context, DragValue, Grid, Theme, Ui, Widget};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, ARROWS_CLOCKWISE, FOLDER, INFO};
use tracing::error;

use crate::{
    app::App,
    render::{camera::Projection, workspace::model::RenderStyle},
    ui::components::{dragger, vec2_dragger, vec3_dragger},
};

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
                .selected_text(match app.config.theme {
                    Theme::Dark => "Dark",
                    Theme::Light => "Light",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.config.theme, Theme::Dark, "Dark");
                    ui.selectable_value(&mut app.config.theme, Theme::Light, "Light");
                });
            ui.end_row();

            ui.horizontal(|ui| {
                ui.label("Render Style");
                ui.label(INFO)
                    .on_hover_text("This setting is really only intended for debugging.");
            });
            ComboBox::from_id_salt("render_style")
                .selected_text(app.config.render_style.name())
                .show_ui(ui, |ui| {
                    for style in RenderStyle::ALL {
                        ui.selectable_value(&mut app.config.render_style, style, style.name());
                    }
                });
            ui.end_row();

            ui.label("Projection");
            ComboBox::from_id_salt("projection")
                .selected_text(app.config.projection.name())
                .show_ui(ui, |ui| {
                    for camera in Projection::ALL {
                        ui.selectable_value(&mut app.config.projection, camera, camera.name());
                    }
                });
            ui.end_row();

            ui.label("Grid Size");
            ui.horizontal(|ui| {
                dragger(ui, "", &mut app.config.grid_size, |x| {
                    x.speed(0.1).range(1.0..=f32::MAX)
                });
                ui.take_available_width();
            });
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.checkbox(&mut app.config.show_normals, "Show Normals");
    ui.add_space(8.0);

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
    });
}
