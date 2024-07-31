use nalgebra::Vector3;

use crate::{half_edge::HalfEdgeMesh, mesh::Mesh};

pub struct LineSupport {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub radius: f32,
}

pub fn generate_line_supports(mesh: &Mesh) -> Vec<LineSupport> {
    let _half_edge_mesh = HalfEdgeMesh::new(mesh);

    vec![]
}

/// Find all points that are both lower than their surrounding points and have down facing normals
fn _detect_point_overhangs(base: &Mesh, mesh: &HalfEdgeMesh) -> Vec<Vector3<f32>> {
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
            // Only add to overhangs if the original point is lower than all connected points
            if this_pos <= origin_pos {
                continue 'outer;
            }
        }

        overhangs.push(vertices[origin_vertex as usize]);
    }

    overhangs
}

fn _detect_face_overhangs(base: &Mesh, _mesh: &HalfEdgeMesh) -> Vec<Vector3<f32>> {
    let mut overhangs = Vec::new();

    for (_idx, face) in base.faces().iter().enumerate() {
        // let face_normal = base.face_normal(idx);
        // if face_normal.z > 0.0 {
        //     continue;
        // }

        let normals = base.normals();
        if normals[face[0] as usize].z > 0.0
            || normals[face[1] as usize].z > 0.0
            || normals[face[2] as usize].z > 0.0
        {
            continue;
        }

        let vertices = base.vertices();
        let center = face
            .iter()
            .fold(Vector3::zeros(), |acc, &v| acc + vertices[v as usize])
            / 3.0;

        if center.z > 5.0 {
            continue;
        }

        overhangs.push(center);
    }

    overhangs
}
