use crate::mesh::Mesh;

use bvh_node::{build_bvh_node, BvhNode};
use nalgebra::Vector3;

mod bounding_box;
mod bvh_node;
mod intersection;

pub struct Bvh {
    root: Option<BvhNode>,
}

impl Bvh {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        if mesh.faces.is_empty() {
            return Self { root: None };
        }

        let face_indices = (0..mesh.faces.len()).collect::<Vec<_>>();
        let root = build_bvh_node(mesh, face_indices);

        Self { root: Some(root) }
    }

    pub fn intersect_plane(
        &self,
        mesh: &Mesh,
        pos: Vector3<f32>,
        normal: Vector3<f32>,
    ) -> Vec<Vector3<f32>> {
        match &self.root {
            Some(root) => {
                let mut out = Vec::new();
                root.intersect_plane(mesh, pos, normal, &mut out);
                out
            }
            None => Vec::new(),
        }
    }
}
