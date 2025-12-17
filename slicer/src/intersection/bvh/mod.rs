use std::sync::Arc;

use crate::mesh::Mesh;

use bvh_node::{build_bvh_node, BvhNode};
use nalgebra::Vector3;
use ordered_float::OrderedFloat;

mod bounding_box;
mod bvh_node;

#[derive(Clone, Default)]
pub struct Bvh {
    nodes: Arc<Vec<BvhNode>>,
}

impl Bvh {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        if mesh.face_count() == 0 {
            return Self::default();
        }

        let mut arena = Vec::with_capacity(mesh.face_count() * 2 - 1);
        let face_indices = (0..mesh.face_count()).collect::<Vec<_>>();

        let root = build_bvh_node(&mut arena, mesh, face_indices);
        arena.push(root);

        Self {
            nodes: Arc::new(arena),
        }
    }

    pub fn intersect_ray(
        &self,
        mesh: &Mesh,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
    ) -> Option<usize> {
        let mut out = Vec::new();
        if let Some(root) = self.nodes.last() {
            root.intersect_ray(&self.nodes, mesh, origin, direction, &mut out);
        }

        out.sort_by_key(|(dist, _face)| OrderedFloat(*dist));
        out.first().map(|(_dist, face)| *face)
    }
}
