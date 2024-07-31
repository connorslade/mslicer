use nalgebra::Vector3;

use crate::{half_edge::HalfEdgeMesh, mesh::Mesh};

pub struct LineSupport {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub radius: f32,
}

pub fn generate_line_supports(mesh: &Mesh) -> Vec<LineSupport> {
    let half_edge_mesh = HalfEdgeMesh::new(mesh);

    let point_overhangs = detect_point_overhangs(&mesh, &half_edge_mesh);
    dbg!(point_overhangs);

    vec![]
}

/// Find all points that are both lower than their surrounding points and have down facing normals
fn detect_point_overhangs(base: &Mesh, mesh: &HalfEdgeMesh) -> Vec<Vector3<f32>> {
    let mut overhangs = Vec::new();

    let vertices = base.vertices();
    let normals = base.normals();

    'outer: for edge in 0..mesh.half_edges().len() {
        let origin_vertex = mesh.vertex(edge as u32);

        // Ignore points that are not on the bottom of the mesh
        let origin_normal = normals[origin_vertex as usize];
        if origin_normal.z > 0.0 {
            continue;
        }

        let origin_pos = vertices[origin_vertex as usize].z;
        for connected in mesh.connected_vertices(edge as u32) {
            let this_pos = vertices[connected as usize].z;
            if this_pos >= origin_pos {
                continue 'outer;
            }
        }

        overhangs.push(vertices[origin_vertex as usize]);
    }

    overhangs
}
