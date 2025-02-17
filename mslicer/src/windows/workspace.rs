use const_format::concatcp;
use eframe::Theme;
use egui::{ComboBox, Context, DragValue, Grid, Ui};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, FOLDER, LAYOUT};
use tracing::error;

use crate::{
    app::App,
    render::pipelines::model::RenderStyle,
    ui::components::{dragger, vec2_dragger, vec3_dragger},
};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.heading("Config");

    ui.horizontal_wrapped(|ui| {
        if ui
            .button(concatcp!(FOLDER, " Open Config Directory"))
            .clicked()
        {
            if let Err(err) = open::that(&app.config_dir) {
                error!("Failed to open config directory: {}", err);
            }
        }

        if ui
            .button(concatcp!(ARROW_COUNTER_CLOCKWISE, " Reset Config"))
            .clicked()
        {
            app.config = Default::default();
        }

        if ui.button(concatcp!(LAYOUT, " Reset UI")).clicked() {
            app.reset_ui();
        }
    });
    ui.add_space(8.0);

    ComboBox::new("render_style", "Render Style")
        .selected_text(app.config.render_style.name())
        .show_ui(ui, |ui| {
            for style in RenderStyle::ALL {
                ui.selectable_value(&mut app.config.render_style, style, style.name());
            }
        });

    ComboBox::new("theme", "Theme")
        .selected_text(match app.config.theme {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut app.config.theme, Theme::Dark, "Dark");
            ui.selectable_value(&mut app.config.theme, Theme::Light, "Light");
        });

    dragger(ui, "Grid Size", &mut app.config.grid_size, |x| x.speed(0.1));

    ui.add_space(16.0);
    ui.heading("Advanced");

    ui.checkbox(&mut app.config.show_normals, "Show Normals");

    ui.collapsing("Camera", |ui| {
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
                ui.add(DragValue::new(&mut app.camera.fov).speed(0.01));
                ui.end_row();

                ui.label("Near");
                ui.add(DragValue::new(&mut app.camera.near));
                ui.end_row();

                ui.label("Far");
                ui.add(DragValue::new(&mut app.camera.far));
                ui.end_row();
            });
    });

    ui.collapsing("Stats", |ui| {
        ui.label(format!(
            "Frame Time: {:.2}ms",
            app.fps.frame_time() * 1000.0
        ));
        ui.label(format!("FPS: {:.2}", 1.0 / app.fps.frame_time()));
    });
}
