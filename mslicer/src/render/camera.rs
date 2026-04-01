use std::{f32::consts::FRAC_PI_2, ops::Neg};

use egui::{PointerButton, Response, Ui};
use nalgebra::{Matrix4, Vector2, Vector3};
use serde::{Deserialize, Serialize};

const EPSILON: f32 = 1e-5;
const NEAR: f32 = 0.1;
const FAR: f32 = 10_000.0;

#[derive(Clone, Debug)]
pub struct Camera {
    pub target: Vector3<f32>,
    pub angle: Vector2<f32>,
    pub distance: f32,
    pub fov: f32,
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Projection {
    Perspective,
    Orthographic,
}

impl Camera {
    pub fn view_projection_matrix(&self, projection: Projection, aspect: f32) -> Matrix4<f32> {
        match projection {
            Projection::Perspective => {
                Matrix4::new_perspective(aspect, self.fov, NEAR, FAR)
                    * Matrix4::look_at_rh(
                        &(self.position(self.distance) + self.target).into(),
                        &self.target.into(),
                        &self.up(),
                    )
            }
            Projection::Orthographic => {
                let height = self.distance * (self.fov / 2.0).sin();
                let width = aspect * height;

                Matrix4::new_orthographic(-width, width, -height, height, -FAR, FAR)
                    * Matrix4::look_at_rh(
                        &(self.position(FAR / 2.0) + self.target).into(),
                        &self.target.into(),
                        &self.up(),
                    )
            }
        }
    }

    // returns ray pos (camera pos) and ray dir
    pub fn hovered_ray(
        &self,
        projection: Projection,
        aspect: f32,
        uv: Vector2<f32>,
    ) -> (Vector3<f32>, Vector3<f32>) {
        let camera_pos = self.position(self.distance) + self.target;
        let pos = 2.0 * Vector2::new(uv.x, 1.0 - uv.y) - Vector2::repeat(1.0);

        let forward = (self.target - camera_pos).normalize();
        let right = forward.cross(&self.up()).normalize();
        let up = right.cross(&forward).normalize();

        match projection {
            Projection::Perspective => {
                let fov_scale = (self.fov * 0.5).tan();
                let uv = pos.component_mul(&Vector2::new(aspect, 1.0)) * fov_scale;
                let dir = (forward + right * uv.x + up * uv.y).normalize();

                (camera_pos, dir)
            }
            Projection::Orthographic => {
                let height = self.distance * (self.fov / 2.0).sin();
                let width = aspect * height;

                let origin = camera_pos + right * pos.x * width + up * pos.y * height;
                (origin, forward)
            }
        }
    }

    pub fn handle_movement(&mut self, response: &Response, ui: &Ui) {
        let shift_down = ui.input(|x| x.modifiers.shift);
        let drag_delta = response.drag_delta() * if shift_down { 0.1 } else { 1.0 };

        if response.dragged_by(PointerButton::Primary) {
            self.angle.x -= drag_delta.x * 0.01;
            self.angle.y += drag_delta.y * 0.01;
        }

        if response.dragged_by(PointerButton::Secondary) {
            let facing = self.position(1.0).neg();
            let right = facing.cross(&self.up()).normalize();
            let up = right.cross(&facing).normalize();
            self.target -= (right * drag_delta.x * 0.1) - (up * drag_delta.y * 0.1);
        }

        if response.hovered() {
            let scroll = ui.input(|x| x.smooth_scroll_delta);
            self.distance = (self.distance - scroll.y * 0.1).max(EPSILON);
        }
    }

    pub fn position(&self, distance: f32) -> Vector3<f32> {
        Vector3::new(
            self.angle.x.cos() * self.angle.y.cos(),
            self.angle.x.sin() * self.angle.y.cos(),
            self.angle.y.sin(),
        ) * distance
    }

    pub fn up(&self) -> Vector3<f32> {
        Vector3::z() * self.angle.y.cos().signum()
    }
}

impl Projection {
    pub const ALL: [Projection; 2] = [Projection::Perspective, Projection::Orthographic];

    pub fn name(&self) -> &'static str {
        match self {
            Projection::Perspective => "Perspective",
            Projection::Orthographic => "Orthographic",
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Vector3::zeros(),
            angle: Vector2::new(-FRAC_PI_2, 0.0),
            distance: 10.0,
            fov: FRAC_PI_2,
        }
    }
}
