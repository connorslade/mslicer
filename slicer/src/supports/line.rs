use std::collections::HashSet;

use nalgebra::{Vector2, Vector3};
use ordered_float::OrderedFloat;
use tracing::info;

use crate::{
    builder::MeshBuilder, half_edge::HalfEdgeMesh,
    intersection::triangle::ray_triangle_intersection, mesh::Mesh,
};

pub struct LineSupportGenerator<'a> {
    config: &'a LineSupportConfig,
    bed_size: Vector3<f32>,
}

pub struct LineSupport {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub radius: f32,
}

pub struct LineSupportConfig {
    /// Support generation
    pub support_radius: f32,
    pub arm_height: f32,
    pub base_radius: f32,
    pub base_height: f32,
    pub support_precision: u32,

    /// Overhang detection
    pub min_angle: f32,
    pub face_support_spacing: f32,
}

impl<'a> LineSupportGenerator<'a> {
    pub fn new(config: &'a LineSupportConfig, bed_size: Vector3<f32>) -> Self {
        Self { config, bed_size }
    }

    pub fn generate_line_supports(&self, mesh: &Mesh) -> (Mesh, Vec<[Vector3<f32>; 2]>) {
        let mut overhangs = Vec::new();
        let half_edge_mesh = HalfEdgeMesh::new(mesh);

        let point_overhangs = self.detect_point_overhangs(mesh, &half_edge_mesh);
        overhangs.extend_from_slice(&point_overhangs);

        let face_overhangs = self.detect_face_overhangs(mesh);
        overhangs.extend_from_slice(&face_overhangs);

        info!(
            "Found {} overhangs {{ face: {}, point: {} }}",
            overhangs.len(),
            face_overhangs.len(),
            point_overhangs.len()
        );

        let mut mesh = MeshBuilder::new();
        let mut debug_points = Vec::new();

        let LineSupportConfig {
            support_radius: sr,
            arm_height: ah,
            base_radius: br,
            base_height: bh,
            support_precision: sp,
            ..
        } = *self.config;

        for [origin, normal] in overhangs {
            let bottom = origin.xy().to_homogeneous();
            let arm_bottom = bottom + Vector3::z() * (origin.z - ah);
            mesh.add_vertical_cylinder(arm_bottom, ah + 0.1, (sr, sr * 0.3), sp);
            mesh.add_vertical_cylinder(bottom, origin.z - ah, (sr, sr), sp);
            mesh.add_vertical_cylinder(bottom, bh, (br, sr), sp);
            debug_points.push([origin, normal]);
        }

        (mesh.build(), debug_points)
    }

    /// Find all points that are both lower than their surrounding points and have down facing normals
    fn detect_point_overhangs(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
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
            if origin_normal.z >= 0.0 {
                continue;
            }

            // Only add to overhangs if the original point is lower than all connected points by one layer
            let origin_pos = mesh.transform(&vertices[origin.origin_vertex as usize]);
            let neighbors = half_edge.connected_vertices(edge as u32);
            if neighbors
                .iter()
                .all(|connected| origin_pos.z < mesh.transform(&vertices[*connected as usize]).z)
            {
                overhangs.push([origin_pos, origin_normal]);
            }
        }

        overhangs
    }

    fn detect_face_overhangs(&self, mesh: &Mesh) -> Vec<[Vector3<f32>; 2]> {
        let mut overhangs = Vec::new();
        let mut out = Vec::new();

        let vertices = mesh.vertices();
        let normals = mesh.normals();

        for (face, normal) in normals.iter().enumerate() {
            let normal = mesh.transform_normal(normal);
            if normal.z >= 0.0 {
                continue;
            }

            let angle = normal.angle(&Vector3::z());
            if angle < self.config.min_angle {
                continue;
            }

            overhangs.push(face);
        }

        let x_count = (self.bed_size.x / self.config.face_support_spacing) as i32;
        let y_count = (self.bed_size.y / self.config.face_support_spacing) as i32;

        for x in 0..x_count {
            for y in 0..y_count {
                let pos = Vector2::new(
                    x as f32 * self.config.face_support_spacing - self.bed_size.x / 2.0,
                    y as f32 * self.config.face_support_spacing - self.bed_size.y / 2.0,
                );

                let mut intersections = Vec::new();
                for &idx in overhangs.iter() {
                    let face = mesh.face(idx);
                    if let Some(intersection) = ray_triangle_intersection(
                        [
                            mesh.transform(&vertices[face[0] as usize]),
                            mesh.transform(&vertices[face[1] as usize]),
                            mesh.transform(&vertices[face[2] as usize]),
                        ],
                        mesh.transform_normal(&normals[idx]),
                        pos.to_homogeneous(),
                        Vector3::z(),
                    ) {
                        intersections.push((intersection, idx));
                    }
                }

                intersections.sort_by_key(|x| OrderedFloat(x.0.z));
                intersections.dedup_by(|a, b| (a.0.z - b.0.z).abs() < 0.1);

                for (intersection, idx) in intersections {
                    let normal = mesh.transform_normal(&normals[idx]);
                    out.push([intersection, normal]);
                }
            }
        }

        out
    }
}

impl Default for LineSupportConfig {
    fn default() -> Self {
        Self {
            support_radius: 0.3,
            arm_height: 1.0,
            base_radius: 1.0,
            base_height: 0.5,
            support_precision: 15,

            min_angle: 3.0,
            face_support_spacing: 1.0,
        }
    }
}
