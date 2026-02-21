use const_format::concatcp;
use egui::{ComboBox, Context, DragValue, Grid, Theme, Ui};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, FOLDER, INFO};
use tracing::error;

use crate::{
    app::App,
    render::workspace::model::RenderStyle,
    ui::components::{dragger, vec2_dragger, vec3_dragger},
};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.heading("Config");

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
            ComboBox::new("theme", "")
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
            ComboBox::new("render_style", "")
                .selected_text(app.config.render_style.name())
                .show_ui(ui, |ui| {
                    for style in RenderStyle::ALL {
                        ui.selectable_value(&mut app.config.render_style, style, style.name());
                    }
                });
            ui.end_row();

            ui.label("Grid Size");
            ui.horizontal(|ui| {
                dragger(ui, "", &mut app.config.grid_size, |x| {
                    x.speed(0.1).range(1.0..=f32::MAX)
                });
                ui.add_space(ui.available_width());
            });
            ui.end_row();
        });

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
