use nalgebra::Vector3;
use ordered_float::OrderedFloat;

use crate::mesh::Mesh;

use super::bounding_box::BoundingBox;

const LEAF_SIZE: usize = 8;

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
    pub fn intersect_ray(
        &self,
        arena: &Vec<BvhNode>,
        mesh: &Mesh,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
        out: &mut Vec<(f32, usize)>,
    ) {
        match self {
            BvhNode::Leaf { faces, bounds } if bounds.intersect_ray(origin, direction) => {
                for face in faces {
                    intersect_ray(mesh, *face, origin, direction, out)
                }
            }
            BvhNode::Node {
                left,
                right,
                bounds,
            } if bounds.intersect_ray(origin, direction) => {
                arena[*left].intersect_ray(arena, mesh, origin, direction, out);
                arena[*right].intersect_ray(arena, mesh, origin, direction, out);
            }
            _ => {}
        }
    }
}

// We can expect there to be a total of 2n - 1 nodes in the final bvh.
// One leaf node for each triangle and n - 1 non-leaf nodes.
pub fn build_bvh_node(
    arena: &mut Vec<BvhNode>,
    mesh: &Mesh,
    mut face_indices: Vec<usize>,
) -> BvhNode {
    let mut bounds = BoundingBox::new();
    for &face in face_indices.iter() {
        bounds.expand_face(mesh, face);
    }

    if face_indices.len() <= LEAF_SIZE {
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

    let left = build_bvh_node(arena, mesh, left_indices.to_vec());
    let left = push_idx(arena, left);

    let right = build_bvh_node(arena, mesh, right_indices.to_vec());
    let right = push_idx(arena, right);

    BvhNode::Node {
        left,
        right,
        bounds,
    }
}

// From https://iquilezles.org/articles/intersectors
// Look into Möller–Trumbore triangle-ray intersection?
fn intersect_ray(
    mesh: &Mesh,
    face_idx: usize,
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    out: &mut Vec<(f32, usize)>,
) {
    let face = mesh.face(face_idx);
    let verts = mesh.vertices();

    let v0 = verts[face[0] as usize];
    let v1 = verts[face[1] as usize];
    let v2 = verts[face[2] as usize];

    let v1v0 = v1 - v0;
    let v2v0 = v2 - v0;
    let rov0 = origin - v0;

    let n = v1v0.cross(&v2v0);
    let q = rov0.cross(&direction);

    let d = direction.dot(&n).recip();
    let u = d * (-q).dot(&v2v0);
    let v = d * q.dot(&v1v0);
    let t = d * (-n).dot(&rov0);

    if !(u < 0.0 || v < 0.0 || (u + v) > 1.0) {
        out.push((t, face_idx));
    }
}
