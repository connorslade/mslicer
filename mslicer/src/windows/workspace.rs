use const_format::concatcp;
use eframe::Theme;
use egui::{ComboBox, Context, Ui};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, FOLDER};
use tracing::error;

use crate::{
    app::App,
    render::pipelines::model::RenderStyle,
    ui::components::{dragger, vec2_dragger, vec3_dragger},
};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.heading("Config");

    ui.horizontal(|ui| {
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
    });
    ui.add_space(8.0);

    ComboBox::new("render_style", "Render Style")
        .selected_text(app.config.render_style.name())
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut app.config.render_style,
                RenderStyle::Normals,
                "Normals",
            );
            ui.selectable_value(&mut app.config.render_style, RenderStyle::Rended, "Rended");
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

    ui.collapsing("Camera", |ui| {
        ui.label("Target");

        vec3_dragger(ui, app.camera.target.as_mut(), |x| x);

        ui.add_space(12.0);
        ui.label("Angle");

        vec2_dragger(ui, app.camera.angle.as_mut(), |x| x);

        ui.add_space(12.0);
        ui.label("Distance");

        dragger(ui, "", &mut app.camera.distance, |x| x.speed(5));

        ui.add_space(12.0);
        ui.label("Misc");

        dragger(ui, "FOV", &mut app.camera.fov, |x| x.speed(0.01));
        dragger(ui, "Near", &mut app.camera.near, |x| x);
        dragger(ui, "Far", &mut app.camera.far, |x| x);
    });
}
