use nalgebra::{Matrix4, Point3, Vector3};

pub struct Camera {
    pub pos: Point3<f32>,
    pub pitch: f32,
    pub yaw: f32,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn view_projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(
            &self.pos,
            &(self.pos
                + Vector3::new(
                    self.pitch.cos() * self.yaw.sin(),
                    self.pitch.sin(),
                    self.pitch.cos() * self.yaw.cos(),
                )),
            &Vector3::new(0.0, 1.0, 0.0),
        );

        let projection = Matrix4::new_perspective(aspect, self.fov, self.near, self.far);

        projection * view
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Point3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,

            fov: std::f32::consts::PI / 2.0,
            near: 0.1,
            far: 100.0,
        }
    }
}
