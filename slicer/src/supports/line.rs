use std::{collections::HashSet, f32::consts::PI};

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
    pub min_angle: f32,
}

impl Default for LineSupportConfig {
    fn default() -> Self {
        Self {
            max_origin_normal_z: 0.0,
            max_neighbor_z_diff: -0.01,
            min_angle: PI / 4.0,
        }
    }
}

pub fn generate_line_supports(mesh: &Mesh, config: &LineSupportConfig) -> Vec<[Vector3<f32>; 2]> {
    // let half_edge_mesh = HalfEdgeMesh::new(mesh);

    // let points = detect_point_overhangs(mesh, &half_edge_mesh, config);
    let points = detect_face_overhangs(mesh, config);
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

fn detect_face_overhangs(mesh: &Mesh, config: &LineSupportConfig) -> Vec<[Vector3<f32>; 2]> {
    let mut overhangs = Vec::new();

    let vertices = mesh.vertices();
    for (face, normal) in mesh.faces().iter().zip(mesh.normals().iter()) {
        let normal = mesh.transform_normal(&normal);
        if normal.z >= config.max_origin_normal_z {
            continue;
        }

        let angle = normal.angle(&Vector3::z());
        if angle < config.min_angle {
            continue;
        }

        let center = face
            .iter()
            .fold(Vector3::zeros(), |acc, &v| acc + vertices[v as usize])
            / 3.0;

        overhangs.push([center, normal]);
    }

    overhangs
}
