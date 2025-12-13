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
    pub fn intersect_plane(
        &self,
        arena: &Vec<BvhNode>,
        mesh: &Mesh,
        pos: Vector3<f32>,
        normal: Vector3<f32>,
        out: &mut Vec<Vector3<f32>>,
    ) {
        match self {
            BvhNode::Leaf { faces, bounds } if bounds.intersect_plane(pos, normal) => {
                for face in faces {
                    intersect_triangle(mesh, *face, pos, normal, out)
                }
            }
            BvhNode::Node {
                left,
                right,
                bounds,
            } if bounds.intersect_plane(pos, normal) => {
                arena[*left].intersect_plane(arena, mesh, pos, normal, out);
                arena[*right].intersect_plane(arena, mesh, pos, normal, out);
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

fn intersect_triangle(
    mesh: &Mesh,
    face: usize,
    point: Vector3<f32>,
    normal: Vector3<f32>,
    out: &mut Vec<Vector3<f32>>,
) {
    let face = mesh.face(face);
    let verts = mesh.vertices();

    let v0 = verts[face[0] as usize];
    let v1 = verts[face[1] as usize];
    let v2 = verts[face[2] as usize];

    let (a, b, c) = (
        (v0 - point).dot(&normal),
        (v1 - point).dot(&normal),
        (v2 - point).dot(&normal),
    );
    let (a_pos, b_pos, c_pos) = (a > 0.0, b > 0.0, c > 0.0);

    let mut push_intersection = |a: f32, b: f32, v0: Vector3<f32>, v1: Vector3<f32>| {
        let (v0, v1) = (mesh.transform(&v0), mesh.transform(&v1));
        let t = a / (a - b);
        let intersection = v0 + t * (v1 - v0);
        out.push(intersection);
    };

    (a_pos ^ b_pos).then(|| push_intersection(a, b, v0, v1));
    (b_pos ^ c_pos).then(|| push_intersection(b, c, v1, v2));
    (c_pos ^ a_pos).then(|| push_intersection(c, a, v2, v0));
}
