use crate::mesh::Mesh;

use bounding_box::BoundingBox;
use nalgebra::Vector3;
use ordered_float::OrderedFloat;
pub mod bounding_box;

pub struct Bvh {
    root: Option<BvhNode>,
}

pub enum BvhNode {
    Leaf {
        face_idx: usize,
        bounds: BoundingBox,
    },
    Node {
        left: Box<BvhNode>,
        right: Box<BvhNode>,
        bounds: BoundingBox,
    },
}

impl Bvh {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        if mesh.faces.is_empty() {
            return Self { root: None };
        }

        let face_indices = (0..mesh.faces.len()).collect::<Vec<_>>();
        let root = build_bvh_node(mesh, face_indices);

        println!("Built BVH {{ node_count: {} }}", root.count_nodes());

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
                let mut out = Vec::with_capacity(mesh.faces.len() * 2 - 1);
                root.intersect_plane(mesh, pos, normal, &mut out);
                out
            }
            None => Vec::new(),
        }
    }
}

impl BvhNode {
    pub fn count_nodes(&self) -> u32 {
        match self {
            BvhNode::Leaf { .. } => 1,
            BvhNode::Node { left, right, .. } => 1 + left.count_nodes() + right.count_nodes(),
        }
    }

    pub fn intersect_plane(
        &self,
        mesh: &Mesh,
        pos: Vector3<f32>,
        normal: Vector3<f32>,
        out: &mut Vec<Vector3<f32>>,
    ) {
        match self {
            BvhNode::Leaf { face_idx, bounds } if bounds.intersect_plane(pos, normal) => {
                intersect_triangle(mesh, *face_idx, pos, normal, out)
            }
            BvhNode::Node {
                left,
                right,
                bounds,
            } if bounds.intersect_plane(pos, normal) => {
                left.intersect_plane(mesh, pos, normal, out);
                right.intersect_plane(mesh, pos, normal, out);
            }
            _ => {}
        }
    }
}

fn build_bvh_node(mesh: &Mesh, mut face_indices: Vec<usize>) -> BvhNode {
    let mut bounds = BoundingBox::new();
    for &face in face_indices.iter() {
        bounds.expand_face(mesh, face);
    }

    if face_indices.len() == 1 {
        return BvhNode::Leaf {
            face_idx: face_indices[0],
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

    let left = build_bvh_node(mesh, left_indices.to_vec());
    let right = build_bvh_node(mesh, right_indices.to_vec());

    BvhNode::Node {
        left: Box::new(left),
        right: Box::new(right),
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
    let face = mesh.faces[face];
    let v0 = mesh.vertices[face[0] as usize];
    let v1 = mesh.vertices[face[1] as usize];
    let v2 = mesh.vertices[face[2] as usize];

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
