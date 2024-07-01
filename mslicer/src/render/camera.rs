use std::ops::Neg;

use egui::{PointerButton, Response, Ui};
use nalgebra::{Matrix4, Vector2, Vector3};

pub struct Camera {
    pub target: Vector3<f32>,
    pub angle: Vector2<f32>,
    pub distance: f32,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn view_projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
        let pos = self.position() + self.target;

        let view = Matrix4::look_at_rh(&pos.into(), &self.target.into(), &Vector3::z_axis());
        let projection = Matrix4::new_perspective(aspect, self.fov, self.near, self.far);

        projection * view
    }

    pub fn handle_movement(&mut self, response: &Response, ui: &Ui) {
        let drag_delta = response.drag_delta();
        if response.dragged_by(PointerButton::Primary) {
            self.angle.x += drag_delta.x * 0.01;
            self.angle.y += drag_delta.y * 0.01;
        }

        if response.dragged_by(PointerButton::Secondary) {
            self.target -=
                self.position().neg().normalize().cross(&Vector3::z_axis()) * drag_delta.x;
            self.target += Vector3::new(0.0, 0.0, drag_delta.y * 0.5);
        }

        let scroll = ui.input(|x| x.smooth_scroll_delta);
        self.distance = (self.distance + scroll.y * 0.1).max(0.0);
    }

    fn position(&self) -> Vector3<f32> {
        Vector3::new(self.angle.x.sin(), self.angle.x.cos(), self.angle.y).normalize()
            * self.distance
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Vector3::zeros(),
            angle: Vector2::zeros(),
            distance: 10.0,

            fov: std::f32::consts::FRAC_PI_2,
            near: 0.1,
            far: 10_000.0,
        }
    }
}
