use nalgebra::Vector3;

use crate::mesh::Mesh;

use super::bvh_node::BvhNode;

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
            BvhNode::Leaf {
                face_idx,
                bounds: _,
            } => intersect_triangle(mesh, *face_idx, pos, normal, out),
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
