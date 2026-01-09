use std::collections::HashSet;

use nalgebra::Vector3;

use crate::{
    half_edge::{HalfEdge, HalfEdgeMesh},
    mesh::Mesh,
};

/// Find all points that are both lower than their surrounding points and have down facing normals
pub fn detect_point_overhangs<T>(
    mesh: &Mesh,
    half_edge: &HalfEdgeMesh,
    map: fn(&HalfEdge, Vector3<f32>, Vector3<f32>) -> T, // half edge, pos, normal
) -> Vec<T> {
    let mut overhangs = Vec::new();
    let mut seen = HashSet::new();

    let vertices = mesh.vertices();
    for edge in 0..half_edge.half_edge_count() {
        let origin = half_edge.get_edge(edge as u32);
        if !seen.insert(origin.origin_vertex) {
            continue;
        }

        // Ignore points that are not on the bottom of the mesh
        let origin_normal = mesh.transform_normal(&mesh.normal(origin.face as usize));
        if origin_normal.z >= 0.0 {
            continue;
        }

        // Only add to overhangs if the original point is lower than all connected points by one layer
        let origin_pos = mesh.transform(&vertices[origin.origin_vertex as usize]);
        let neighbors = half_edge.connected_vertices(edge as u32);
        if (neighbors.iter())
            .all(|connected| origin_pos.z < mesh.transform(&vertices[*connected as usize]).z)
        {
            overhangs.push(map(origin, origin_pos, origin_normal));
        }
    }

    overhangs
}
