use egui::{Key, PointerButton, Response, Ui};
use nalgebra::{Matrix4, Vector3};

pub struct Camera {
    pub pos: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn view_projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(
            &self.pos.into(),
            &(self.direction() + self.pos).into(),
            &Vector3::z_axis(),
        );
        let projection = Matrix4::new_perspective(aspect, self.fov, self.near, self.far);

        projection * view
    }

    pub fn direction(&self) -> Vector3<f32> {
        Vector3::new(
            self.pitch.cos() * self.yaw.cos(),
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
        )
    }

    pub fn handle_movement(&mut self, response: &Response, ui: &Ui) {
        let drag_delta = response.drag_delta();
        if response.dragged_by(PointerButton::Primary) {
            self.pitch -= drag_delta.y * 0.01;
            self.yaw -= drag_delta.x * 0.01;
        }

        let direction = self.direction();
        let (w, a, s, d, space, shift, ctrl) = ui.input(|x| {
            (
                x.key_down(Key::W),
                x.key_down(Key::A),
                x.key_down(Key::S),
                x.key_down(Key::D),
                x.key_down(Key::Space),
                x.modifiers.shift,
                x.modifiers.ctrl,
            )
        });

        let speed = if ctrl { 0.4 } else { 0.2 };

        if w {
            self.pos += direction * speed;
        }

        if s {
            self.pos -= direction * speed;
        }

        if d {
            self.pos += direction.cross(&Vector3::z()) * speed;
        }

        if a {
            self.pos -= direction.cross(&Vector3::z()) * speed;
        }

        if space {
            self.pos += Vector3::z() * speed;
        }

        if shift {
            self.pos -= Vector3::z() * speed;
        }

        let scroll = ui.input(|x| x.smooth_scroll_delta);
        self.pos += direction * scroll.y * 0.1;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vector3::zeros(),
            pitch: 0.0,
            yaw: 0.0,

            fov: std::f32::consts::PI / 2.0,
            near: 0.1,
            far: 100.0,
        }
    }
}
