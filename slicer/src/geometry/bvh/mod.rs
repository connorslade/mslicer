use std::sync::Arc;

use common::progress::Progress;
use nalgebra::Vector3;

use super::{Hit, Ray};
use crate::{
    geometry::{Primitive, primitive},
    mesh::{Mesh, MeshInner},
};
use bvh_node::{BvhNode, build_bvh_node};

mod bounding_box;
mod bvh_node;

const LEAF_SIZE: usize = 8;

#[derive(Clone, Default)]
pub struct Bvh {
    nodes: Arc<Vec<BvhNode>>,
}

impl Bvh {
    pub fn build(mesh: &Arc<MeshInner>, progress: Progress) -> Self {
        let faces = mesh.faces.len();
        progress.set_total(faces as u64);

        if faces == 0 {
            return Self::default();
        }

        let mut arena = Vec::new();
        let face_indices = (0..faces).collect::<Vec<_>>();

        let root = build_bvh_node(&mut arena, mesh, &progress, face_indices);
        arena.push(root);

        progress.set_finished();
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
