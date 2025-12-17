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

    // returns ray pos (camera pos) and ray dir
    pub fn hovered_ray(&self, aspect: f32, (u, v): (f32, f32)) -> (Vector3<f32>, Vector3<f32>) {
        let camera_pos = self.position() + self.target;

        // Convert screen coordinates (u, v) in [0, 1] to normalized device coords,
        // with x in [-1, 1] (left to right) and y in [-1, 1] (bottom to top).
        // Many UI systems have v increasing downward, so we flip v here.
        let x_ndc = 2.0 * u - 1.0;
        let y_ndc = 1.0 - 2.0 * v;

        // Image plane half-height at unit distance from the camera.
        let tan_half_fov = (self.fov * 0.5).tan();

        // Camera basis in world space.
        // forward points from camera position toward the target.
        let forward = (self.target - camera_pos).normalize();
        let right = forward.cross(&Vector3::z()).normalize();
        let up = right.cross(&forward).normalize();

        // Compute a point on the image plane in world space and form the ray.
        let image_plane_point = camera_pos
            + forward
            + right * (x_ndc * aspect * tan_half_fov)
            + up * (y_ndc * tan_half_fov);
        let dir = (image_plane_point - camera_pos).normalize();

        (camera_pos, dir)
    }

    pub fn handle_movement(&mut self, response: &Response, ui: &Ui) {
        let shift_down = ui.input(|x| x.modifiers.shift);
        let drag_delta = response.drag_delta() * if shift_down { 0.1 } else { 1.0 };

        if response.dragged_by(PointerButton::Primary) {
            self.angle.x += drag_delta.x * 0.01;
            self.angle.y = (self.angle.y + drag_delta.y * 0.01)
                .clamp(-FRAC_PI_2 + EPSILON, FRAC_PI_2 - EPSILON);
        }

        if response.dragged_by(PointerButton::Secondary) {
            let facing = Vector3::new(self.angle.x.sin(), self.angle.x.cos(), self.angle.y.tan())
                .normalize()
                .neg();

            let right = facing.cross(&Vector3::z()).normalize();
            let up = right.cross(&facing).normalize();
            self.target -= (right * drag_delta.x * 0.1) - (up * drag_delta.y * 0.1);
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
