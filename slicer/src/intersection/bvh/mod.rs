use crate::mesh::Mesh;

use bvh_node::{build_bvh_node, BvhNode};
use nalgebra::Vector3;

mod bounding_box;
mod bvh_node;

pub struct Bvh {
    /// The root node is the last one
    nodes: Vec<BvhNode>,
}

impl Bvh {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        if mesh.face_count() == 0 {
            return Self { nodes: Vec::new() };
        }

        let mut arena = Vec::with_capacity(mesh.face_count() * 2 - 1);
        let face_indices = (0..mesh.face_count()).collect::<Vec<_>>();

        let root = build_bvh_node(&mut arena, mesh, face_indices);
        arena.push(root);

        Self { nodes: arena }
    }

    pub fn intersect_plane(
        &self,
        mesh: &Mesh,
        pos: Vector3<f32>,
        normal: Vector3<f32>,
    ) -> Vec<Vector3<f32>> {
        if let Some(root) = self.nodes.last() {
            let mut out = Vec::new();
            root.intersect_plane(&self.nodes, mesh, pos, normal, &mut out);
            out
        } else {
            Vec::new()
        }
    }
}
