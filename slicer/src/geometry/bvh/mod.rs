use std::sync::Arc;

use nalgebra::Vector3;

use super::{Hit, Ray};
use crate::{
    geometry::{Primitive, primitive},
    mesh::Mesh,
};
use bvh_node::{BvhNode, build_bvh_node};

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

    fn intersect<Type: Primitive>(&self, mesh: &Mesh, ray: Ray) -> Option<Hit> {
        let mut hit = Hit::default();
        if let Some(root) = self.nodes.last() {
            root.intersect::<Type>(&self.nodes, mesh, ray, &mut hit);
        }

        (hit.face != usize::MAX).then_some(hit)
    }

    pub fn closest(&self, mesh: &Mesh, point: Vector3<f32>) -> Option<Hit> {
        self.nodes.last().map(|root| {
            let mut hit = Hit::default();
            root.closest(&self.nodes, mesh, point, &mut hit);
            hit.t = hit.t.sqrt();
            hit
        })
    }

    pub fn intersect_ray(
        &self,
        mesh: &Mesh,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
    ) -> Option<Hit> {
        self.intersect::<primitive::Ray>(mesh, Ray { origin, direction })
    }

    pub fn intersect_segment(&self, mesh: &Mesh, a: Vector3<f32>, b: Vector3<f32>) -> Option<Hit> {
        self.intersect::<primitive::Segment>(
            mesh,
            Ray {
                origin: a,
                direction: b - a,
            },
        )
    }
}
