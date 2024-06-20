use nalgebra::{Matrix4, Point3, Vector3};

const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn view_projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
        let fov = self.fovy * std::f32::consts::PI / 180.0;

        let view = Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = Matrix4::new_perspective(aspect, fov, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Point3::new(0.0, -50.0, 5.0),
            target: Point3::new(0.0, 50.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            fovy: 25.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}
