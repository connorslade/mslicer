use std::{f32::consts::FRAC_PI_2, ops::Neg};

use egui::{PointerButton, Response, Ui};
use nalgebra::{Matrix4, Vector2, Vector3};

const EPSILON: f32 = 1e-5;

#[derive(Clone, Debug)]
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
            self.angle.y = (self.angle.y + drag_delta.y * 0.01)
                .clamp(-FRAC_PI_2 + EPSILON, FRAC_PI_2 - EPSILON);
        }

        if response.dragged_by(PointerButton::Secondary) {
            let facing = Vector3::new(self.angle.x.sin(), self.angle.x.cos(), self.angle.y.tan())
                .normalize()
                .neg();
            self.target -= facing.cross(&Vector3::z_axis()) * drag_delta.x;
            self.target -= facing.cross(&Vector3::x_axis()) * drag_delta.y;
        }

        if response.hovered() {
            let scroll = ui.input(|x| x.smooth_scroll_delta);
            self.distance = (self.distance - scroll.y * 0.1).max(EPSILON);
        }
    }

    pub fn position(&self) -> Vector3<f32> {
        Vector3::new(self.angle.x.sin(), self.angle.x.cos(), self.angle.y.tan()).normalize()
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
