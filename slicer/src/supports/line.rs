use std::collections::HashSet;

use nalgebra::Vector3;

use crate::{half_edge::HalfEdgeMesh, mesh::Mesh};

pub struct LineSupport {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub radius: f32,
}

pub struct LineSupportConfig {
    pub max_origin_normal_z: f32,
    pub max_neighbor_z_diff: f32,
}

impl Default for LineSupportConfig {
    fn default() -> Self {
        Self {
            max_origin_normal_z: -0.5,
            max_neighbor_z_diff: 0.05,
        }
    }
}

pub fn generate_line_supports(mesh: &Mesh, config: &LineSupportConfig) -> Vec<[Vector3<f32>; 2]> {
    let half_edge_mesh = HalfEdgeMesh::new(mesh);

    let points = detect_point_overhangs(mesh, &half_edge_mesh, config);
    println!("Found {} overhangs", points.len());

    points
}

/// Find all points that are both lower than their surrounding points and have down facing normals
fn detect_point_overhangs(
    mesh: &Mesh,
    half_edge: &HalfEdgeMesh,
    config: &LineSupportConfig,
) -> Vec<[Vector3<f32>; 2]> {
    let mut overhangs = Vec::new();
    let mut seen = HashSet::new();

    let vertices = mesh.vertices();
    let normals = mesh.normals();

    for edge in 0..half_edge.half_edge_count() {
        let origin = half_edge.get_edge(edge as u32);
        if !seen.insert(origin.origin_vertex) {
            continue;
        }

        // Ignore points that are not on the bottom of the mesh
        let origin_normal = mesh.transform_normal(&normals[origin.face as usize]);
        if origin_normal.z >= config.max_origin_normal_z {
            continue;
        }

        // Only add to overhangs if the original point is lower than all connected points by one layer
        let origin_pos = mesh.transform(&vertices[origin.origin_vertex as usize]);
        let neighbors = half_edge.connected_vertices(edge as u32);
        if neighbors.iter().all(|connected| {
            (origin_pos.z - mesh.transform(&vertices[*connected as usize]).z)
                <= config.max_neighbor_z_diff
        }) {
            overhangs.push([origin_pos, origin_normal]);
        }
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

        overhangs.push(center);
    }

    overhangs
}
