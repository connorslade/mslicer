use std::sync::Arc;

use common::progress::Progress;
use nalgebra::Vector3;
use ordered_float::OrderedFloat;

use crate::{
    geometry::{
        Hit, Primitive,
        bvh::{LEAF_SIZE, Ray},
        triangle::{closest_point, triangle_intersection},
    },
    mesh::{Mesh, MeshInner},
};

use super::bounding_box::BoundingBox;

pub enum BvhNode {
    Leaf {
        faces: Box<[usize]>,
        bounds: BoundingBox,
    },
    Node {
        left: usize,
        right: usize,
        bounds: BoundingBox,
    },
}

impl BvhNode {
    fn bounds(&self) -> &BoundingBox {
        match self {
            BvhNode::Leaf { bounds, .. } | BvhNode::Node { bounds, .. } => bounds,
        }
    }

    pub fn intersect<Type: Primitive>(
        &self,
        arena: &[BvhNode],
        mesh: &Mesh,
        ray: Ray,
        out: &mut Hit,
    ) {
        if !(self.bounds().intersect::<Type>(ray))
            .map(|t| t < out.t)
            .unwrap_or_default()
        {
            return;
        };

        match self {
            BvhNode::Leaf { faces, .. } => faces.iter().for_each(|&face| {
                if let Some(hit) = triangle_intersection::<Type>(mesh, face, ray)
                    && hit.t.abs() < out.t.abs()
                {
                    *out = hit;
                }
            }),
            BvhNode::Node { left, right, .. } => {
                let left = (arena[*left].bounds().intersect::<Type>(ray))
                    .and_then(|t| (t <= out.t).then_some((t, *left)));
                let right = (arena[*right].bounds().intersect::<Type>(ray))
                    .and_then(|t| (t <= out.t).then_some((t, *right)));

                if let (Some(a), Some(b)) = (left, right) {
                    let (first, second) = if a.0 < b.0 { (a, b) } else { (b, a) };
                    arena[first.1].intersect::<Type>(arena, mesh, ray, out);
                    if second.0 <= out.t {
                        arena[second.1].intersect::<Type>(arena, mesh, ray, out);
                    }
                } else if let Some((_, child)) = left.or(right) {
                    arena[child].intersect::<Type>(arena, mesh, ray, out);
                }
            }
        }
    }

    pub fn closest(&self, arena: &[BvhNode], mesh: &Mesh, point: Vector3<f32>, out: &mut Hit) {
        if self.bounds().distance(point) >= out.t {
            return;
        }

        match self {
            BvhNode::Leaf { faces, .. } => faces.iter().for_each(|&face| {
                let position = closest_point(mesh, face, point);
                let t = (position - point).magnitude_squared();
                if t < out.t {
                    *out = Hit { position, t, face };
                }
            }),
            BvhNode::Node { left, right, .. } => {
                let (left, right) = (&arena[*left], &arena[*right]);
                let dist_left = left.bounds().distance(point);
                let dist_right = right.bounds().distance(point);

                if dist_left < dist_right {
                    left.closest(arena, mesh, point, out);
                    right.closest(arena, mesh, point, out);
                } else {
                    right.closest(arena, mesh, point, out);
                    left.closest(arena, mesh, point, out);
                }
            }
        }
    }
}

// We can expect there to be a total of 2n - 1 nodes in the final bvh.
// One leaf node for each triangle and n - 1 non-leaf nodes.
pub fn build_bvh_node(
    arena: &mut Vec<BvhNode>,
    mesh: &Arc<MeshInner>,
    progress: &Progress,
    mut face_indices: Vec<usize>,
) -> BvhNode {
    let mut bounds = BoundingBox::new();
    for &face in face_indices.iter() {
        bounds.expand_face(mesh, face);
    }

    if face_indices.len() <= LEAF_SIZE {
        progress.add_complete(face_indices.len() as u64);
        return BvhNode::Leaf {
            faces: face_indices.into_boxed_slice(),
            bounds,
        };
    }

    let longest_axis = bounds.longest_axis();
    face_indices.sort_by_cached_key(|&x| {
        let mut bounds = BoundingBox::new();
        bounds.expand_face(mesh, x);
        OrderedFloat(bounds.center()[longest_axis])
    });

    let (left_indices, right_indices) = face_indices.split_at(face_indices.len() / 2);

    let push_idx = |arena: &mut Vec<BvhNode>, val| {
        arena.push(val);
        arena.len() - 1
    };

    let left = build_bvh_node(arena, mesh, progress, left_indices.to_vec());
    let left = push_idx(arena, left);

    let right = build_bvh_node(arena, mesh, progress, right_indices.to_vec());
    let right = push_idx(arena, right);

    BvhNode::Node {
        left,
        right,
        bounds,
    }
}
