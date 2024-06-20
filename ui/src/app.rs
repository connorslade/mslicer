use egui::{CentralPanel, Frame, Sense, TopBottomPanel};
use egui_wgpu::Callback;

use crate::{camera::Camera, workspace::WorkspaceRenderCallback};

pub struct App {
    pub camera: Camera,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                self.camera.eye.x += drag_delta.x;
                self.camera.eye.y -= drag_delta.y;

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
