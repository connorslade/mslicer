use std::{mem, sync::Arc};

use common::progress::Progress;
use nalgebra::Vector3;
use ordered_float::OrderedFloat;

use crate::geometry::{
    Hit, Primitive, Ray,
    bounds::Bounds3D,
    mesh::{Mesh, MeshInner},
    primitive,
    triangle::{closest_point, triangle_intersection},
};

const LEAF_SIZE: usize = 8;

#[derive(Clone, Default)]
pub struct Bvh {
    nodes: Arc<Vec<BvhNode>>,
}

pub enum BvhNode {
    Leaf {
        faces: Box<[usize]>,
        bounds: Bounds3D,
    },
    Node {
        left: usize,
        right: usize,
        bounds: Bounds3D,
    },
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

    pub fn memory_size(&self) -> usize {
        self.nodes.capacity() * mem::size_of::<BvhNode>()
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
            root.closest(&self.nodes, mesh, mesh.inv_transform(&point), &mut hit);
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
        let (origin, direction) = (
            mesh.inv_transform(&origin),
            mesh.inv_transform_normal(&direction),
        );

        self.intersect::<primitive::Ray>(mesh, Ray { origin, direction })
    }

    pub fn intersect_segment(&self, mesh: &Mesh, a: Vector3<f32>, b: Vector3<f32>) -> Option<Hit> {
        let (a, b) = (mesh.inv_transform(&a), mesh.inv_transform(&b));
        let ray = Ray {
            origin: a,
            direction: b - a,
        };

        self.intersect::<primitive::Segment>(mesh, ray)
    }
}

impl BvhNode {
    fn bounds(&self) -> &Bounds3D {
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
                let triangle = (mesh.face_verts_raw(face), mesh.normal(face));
                if let Some(hit) = triangle_intersection::<Type>(triangle, ray)
                    && hit.0.abs() < out.t.abs()
                {
                    *out = Hit {
                        position: hit.1,
                        t: hit.0,
                        face,
                    };
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
                let position = closest_point(mesh.face_verts_raw(face), point);
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
    let mut bounds = Bounds3D::empty();
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
        let mut bounds = Bounds3D::empty();
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
