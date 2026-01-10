use std::sync::Arc;

use crate::mesh::Mesh;

use bvh_node::{BvhNode, build_bvh_node};
use nalgebra::Vector3;

mod bounding_box;
mod bvh_node;

#[derive(Clone, Default)]
pub struct Bvh {
    nodes: Arc<Vec<BvhNode>>,
}

#[derive(Clone, Copy)]
struct Ray {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
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

    fn intersect<const SEGMENT: bool>(&self, mesh: &Mesh, ray: Ray) -> Option<usize> {
        let mut out = (f32::MAX, usize::MAX);
        if let Some(root) = self.nodes.last() {
            root.intersect::<SEGMENT>(&self.nodes, mesh, ray, &mut out);
        }

        (out.1 != usize::MAX).then_some(out.1)
    }

    pub fn intersect_ray(
        &self,
        mesh: &Mesh,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
    ) -> Option<usize> {
        self.intersect::<false>(mesh, Ray { origin, direction })
    }

    pub fn intersect_segment(
        &self,
        mesh: &Mesh,
        a: Vector3<f32>,
        b: Vector3<f32>,
    ) -> Option<usize> {
        self.intersect::<true>(
            mesh,
            Ray {
                origin: a,
                direction: b - a,
            },
        )
    }
}
