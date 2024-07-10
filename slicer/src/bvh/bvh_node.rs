use ordered_float::OrderedFloat;

use crate::mesh::Mesh;

use super::bounding_box::BoundingBox;

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

// We can expect there to be a total of 2n - 1 nodes in the final bvh.
// One leaf node for each triangle and n - 1 non-leaf nodes.
pub fn build_bvh_node(mesh: &Mesh, mut face_indices: Vec<usize>) -> BvhNode {
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
