use nalgebra::{Matrix4, Vector3};

use crate::geometry::{bounds::Bounds3D, mesh::Mesh};

pub struct CachedBounds {
    hull: Box<[Vector3<f32>]>,

    transform: Matrix4<f32>,
    bounds: Bounds3D,
}

impl CachedBounds {
    pub fn new(mesh: &Mesh) -> Self {
        todo!()
    }
}
