use eframe::{Frame, Theme};
use egui::{ComboBox, Context, Visuals, Window};

use crate::{
    app::App,
    components::{dragger, vec2_dragger, vec3_dragger},
    render::pipelines::model::RenderStyle,
};

pub fn ui(app: &mut App, ctx: &Context, _frame: &mut Frame) {
    Window::new("Workspace")
        .open(&mut app.windows.show_workspace)
        .default_width(0.0)
        .show(ctx, |ui| {
            ComboBox::new("render_style", "Render Style")
                .selected_text(app.render_style.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.render_style, RenderStyle::Normals, "Normals");
                    ui.selectable_value(&mut app.render_style, RenderStyle::Rended, "Rended");
                });

            let last_theme = app.theme;
            ComboBox::new("theme", "Theme")
                .selected_text(match app.theme {
                    Theme::Dark => "Dark",
                    Theme::Light => "Light",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.theme, Theme::Dark, "Dark");
                    ui.selectable_value(&mut app.theme, Theme::Light, "Light");
                });

            if last_theme != app.theme {
                match app.theme {
                    Theme::Dark => ctx.set_visuals(Visuals::dark()),
                    Theme::Light => ctx.set_visuals(Visuals::light()),
                }
            }

            dragger(ui, "Grid Size", &mut app.grid_size, |x| x.speed(0.1));

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
        });
}
