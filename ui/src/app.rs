use egui::{CentralPanel, Frame, PointerButton, Sense, TopBottomPanel};
use egui_wgpu::Callback;

use crate::{camera::Camera, workspace::WorkspaceRenderCallback};

pub struct App {
    pub camera: Camera,

    target_distance: f32,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let scroll = ctx.input(|i| i.smooth_scroll_delta);
        self.target_distance -= scroll.y * 0.1;
        self.camera.pos.z = self.target_distance;

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("mslicer");
                ui.separator();
                if ui.button("Organize windows").clicked() {
                    ui.ctx().memory_mut(|mem| mem.reset_areas());
                }
            });
        });

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

                let drag_delta = response.drag_delta();
                if response.dragged_by(PointerButton::Primary) {
                    self.camera.yaw -= drag_delta.x * 0.01;
                    self.camera.pitch += drag_delta.y * 0.01;
                }

                if response.dragged_by(PointerButton::Secondary) {
                    self.camera.pos.x -= drag_delta.x * 0.01;
                    self.camera.pos.y += drag_delta.y * 0.01;
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
        }
    }
}
