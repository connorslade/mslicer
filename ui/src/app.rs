use egui::{CentralPanel, Frame, Sense, Slider, TopBottomPanel, Window};
use egui_wgpu::Callback;
use nalgebra::Point3;

use crate::{camera::Camera, workspace::WorkspaceRenderCallback};

pub struct App {
    pub camera: Camera,

    target_distance: f32,
    target_pitch: f32,
    target_yaw: f32,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let scroll = ctx.input(|i| i.smooth_scroll_delta);
        self.target_distance += scroll.y * 0.1;

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("mslicer");
                ui.separator();
                if ui.button("Organize windows").clicked() {
                    ui.ctx().memory_mut(|mem| mem.reset_areas());
                }
            });
        });

        Window::new("debug").show(ctx, |ui| {
            ui.add(Slider::new(&mut self.camera.eye.x, -50.0..=50.0).text("eye.x"));
            ui.add(Slider::new(&mut self.camera.eye.y, -50.0..=50.0).text("eye.y"));
            ui.add(Slider::new(&mut self.camera.eye.z, -50.0..=50.0).text("eye.z"));
        });

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

                if response.dragged() {
                    let drag_delta = response.drag_delta();
                    self.target_yaw -= drag_delta.x * 0.01;
                    self.target_pitch += drag_delta.y * 0.01;

                    self.camera.eye = Point3::new(
                        self.target_distance * self.target_pitch.cos() * self.target_yaw.sin(),
                        self.target_distance * self.target_pitch.sin(),
                        self.target_distance * self.target_pitch.cos() * self.target_yaw.cos(),
                    );
                }

                let callback = Callback::new_paint_callback(
                    rect,
                    WorkspaceRenderCallback {
                        transform: self
                            .camera
                            .view_projection_matrix(rect.width() / rect.height()),
                    },
                );
                ui.painter().add(callback);
            });
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            target_distance: 50.0,
            target_pitch: 0.0,
            target_yaw: 0.0,
        }
    }
}
