use eframe::Frame;
use egui::{ComboBox, Context, Window};

use crate::{
    app::App,
    components::{dragger, vec2_dragger, vec3_dragger},
    workspace::RenderStyle,
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
            ui.collapsing("Camera", |ui| {
                ui.label("Position");

                vec3_dragger(ui, app.camera.pos.as_mut());

                ui.add_space(12.0);
                ui.label("Target");

                let mut looking = [app.camera.pitch, app.camera.yaw];
                vec2_dragger(ui, &mut looking);
                app.camera.pitch = looking[0];
                app.camera.yaw = looking[1];

                ui.add_space(12.0);
                ui.label("Misc");

                dragger(ui, "FOV", &mut app.camera.fov, |x| x.speed(0.01));
                dragger(ui, "Near", &mut app.camera.near, |x| x);
                dragger(ui, "Far", &mut app.camera.far, |x| x);
            });
        });
}
