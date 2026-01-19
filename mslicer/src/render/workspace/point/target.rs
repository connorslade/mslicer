use std::slice;

use nalgebra::{Vector3, Vector4};

use crate::{
    app::App,
    render::workspace::point::{Point, PointGenerator},
};

const UNDEFINED: Vector3<f32> = Vector3::new(f32::NAN, f32::NAN, f32::NAN);

pub struct TargetPointDispatch {
    point: Point,
}

impl TargetPointDispatch {
    pub fn new() -> Self {
        Self {
            point: Point {
                position: UNDEFINED,
                radius: 1.0,
                color: Vector4::new(1.0, 0.0, 0.0, 0.25),
            },
        }
    }
}

impl PointGenerator for TargetPointDispatch {
    fn generate_points(&mut self, app: &mut App) {
        let is_moving = app.state.workspace.is_moving;
        self.point.position = [UNDEFINED, app.camera.target][is_moving as usize];
    }

    fn points(&self) -> &[Point] {
        if self.point.position.x.is_nan() {
            &[]
        } else {
            slice::from_ref(&self.point)
        }
    }
}
