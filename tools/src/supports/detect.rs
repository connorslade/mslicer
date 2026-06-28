use std::collections::HashSet;

use common::container::ArrayCluster;
use nalgebra::Vector3;
use slicer::{
    geometry::{Ray, primitive, triangle::triangle_intersection},
    half_edge::HalfEdgeMesh,
    mesh::Mesh,
};

use crate::supports::{SupportGenerator, SupportPlacement, quazirandom::quazirandom_rect_2d};

impl<'a> SupportGenerator<'a> {
    pub fn overhanging_faces(&self, mesh: &Mesh) -> Vec<(usize, Vector3<f32>)> {
        let max_angle = self.config.max_angle.to_radians();
        let mut overhangs = Vec::new();

        for face in 0..mesh.face_count() {
            let normal = mesh.transform_normal(&mesh.normal(face));

            let angle = (-normal.z).acos();
            (angle <= max_angle).then(|| overhangs.push((face, normal)));
        }

        overhangs
    }

    // Find edges that border two faces with different angles, at least one of
    // the faces must be overhanging.
    pub fn overhanging_edges(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
        overhangs: &[(usize, Vector3<f32>)],
    ) -> Vec<HashSet<u32>> {
        let mut cluster = ArrayCluster::new(mesh.vertex_count());

        for (face, normal) in overhangs.iter() {
            let verts = mesh.face(*face);
            for i in 0..3 {
                let Some(edge) = half_edge.edge_for((verts[i], verts[(i + 1) % 3])) else {
                    continue;
                };

                let Some(twin) = edge.twin else { continue }; // ✌️
                let neighbor = half_edge.get_edge(twin).face as usize;

                let neighbor_normal = mesh.transform_normal(&mesh.normal(neighbor));
                let angle_diff = normal.angle(&neighbor_normal);

                // 0.1 rad ≈ 5°
                if angle_diff > self.config.edge_angle_delta {
                    cluster.union(edge.origin_vertex, edge.vertex);
                }
            }
        }

        cluster
            .clusters()
            .map(|x| x.iter().copied().collect::<HashSet<_>>())
            .collect()
    }

    pub fn place_face_supports(
        &self,
        mesh: &Mesh,
        overhangs: &[(usize, Vector3<f32>)],
    ) -> Vec<SupportPlacement> {
        let mut out = Vec::new();
        let bed_size = self.bed_size.xy().map(|x| x.raw());

        for pos in quazirandom_rect_2d(bed_size, self.config.face_support_spacing) {
            let pos = pos - bed_size / 2.0;
            let mut intersections = Vec::new();
            for (idx, _angle) in overhangs.iter() {
                let ray = Ray {
                    origin: mesh.inv_transform(&pos.to_homogeneous()),
                    direction: mesh.inv_transform_normal(&Vector3::z()),
                };
                let face = (mesh.face_verts_raw(*idx), mesh.normal(*idx));
                if let Some(mut intersection) = triangle_intersection::<primitive::Ray>(face, ray) {
                    intersection.1 = mesh.transform(&intersection.1);
                    intersections.push(SupportPlacement {
                        point: intersection.1,
                        normal: face.1,
                    });
                }
            }

            out.extend(intersections);
        }

        out
    }

    pub fn place_edge_supports(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
        overhangs: &[(usize, Vector3<f32>)],
    ) -> Vec<SupportPlacement> {
        let edge_runs = self.overhanging_edges(mesh, half_edge, overhangs);
        let mut out = Vec::new();

        for run in edge_runs.iter() {
            let mut seen = HashSet::new();
            let start = *run.iter().next().unwrap();
            let mut stack = vec![(start, 0.0)];

            while let Some((vertex, t)) = stack.pop() {
                let start_edge = half_edge.incident_edge(vertex).unwrap();
                let mut edge = start_edge;

                while {
                    if run.contains(&edge.vertex) {
                        let canonical = (
                            edge.origin_vertex.min(edge.vertex),
                            edge.origin_vertex.max(edge.vertex),
                        );

                        if seen.insert(canonical) {
                            let [a, b] = [edge.origin_vertex, edge.vertex]
                                .map(|x| mesh.transform(&mesh.vertices()[x as usize]));

                            let vector = b - a;
                            let len = vector.magnitude();
                            let unit = vector / len;

                            let normal =
                                (mesh.transform_normal(&mesh.normal(edge.face as usize))
                                    + mesh.transform_normal(&mesh.normal(
                                        half_edge.get_edge(edge.twin.unwrap()).face as usize,
                                    )))
                                .normalize();

                            let mut t = t;
                            while t < len {
                                let point = a + unit * t;
                                out.push(SupportPlacement { point, normal });
                                t += self.config.edge_support_spacing;
                            }

                            stack.push((edge.vertex, t - len));
                        }
                    }

                    edge = half_edge.get_edge(half_edge.get_edge(edge.twin.unwrap()).next);
                    edge != start_edge
                } {}
            }
        }

        out
    }

    /// Find all points that are both lower than their surrounding points and have down facing normals
    pub fn place_point_supports(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
    ) -> Vec<SupportPlacement> {
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
            if (neighbors.iter()).all(|connected| {
                origin_pos.z < mesh.transform(&vertices[connected.vertex as usize]).z
            }) {
                let point = mesh.transform(&mesh.vertices()[origin.vertex as usize]);
                let normal = (neighbors.iter())
                    .map(|x| mesh.normal(x.face as usize))
                    .sum::<Vector3<_>>()
                    .normalize();

                overhangs.push(SupportPlacement { point, normal });
            }
        }

        overhangs
    }
}
