use std::collections::HashSet;

use common::container::Clusters;
use nalgebra::{Vector2, Vector3};
use ordered_float::OrderedFloat;
use slicer::{
    geometry::{Ray, primitive, triangle::triangle_intersection},
    half_edge::{HalfEdge, HalfEdgeMesh},
    mesh::Mesh,
};

use crate::supports::SupportGenerator;

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

impl<'a> SupportGenerator<'a> {
    pub fn overhanging_faces(&self, mesh: &Mesh) -> Vec<(usize, Vector3<f32>)> {
        let mut overhangs = Vec::new();
        for face in 0..mesh.face_count() {
            let normal = mesh.transform_normal(&mesh.normal(face));

            let angle = normal.angle(&-Vector3::z());
            if angle > self.config.max_angle {
                continue;
            }

            overhangs.push((face, normal));
        }

        overhangs
    }

    // Find edges that border two faces with different angles, at least one of
    // the faces must be overhanging.
    pub fn overhanging_edges(&self, mesh: &Mesh, half_edge: &HalfEdgeMesh) -> Vec<Vec<u32>> {
        let mut cluster = Clusters::default();
        let mut seen = HashSet::new();
        let overhangs = self.overhanging_faces(mesh); // todo: don't call twice

        println!("overhanging faces: {}", overhangs.len());

        for (i, (face, normal)) in overhangs.iter().enumerate() {
            println!("{:.1}%", 100.0 * i as f32 / overhangs.len() as f32);

            let verts = mesh.face(*face);
            for i in 0..3 {
                let Some(edge) = half_edge.edge_for((verts[i], verts[(i + 1) % 3])) else {
                    continue;
                };

                let Some(twin) = edge.twin else { continue }; // ✌️
                let neighbor = half_edge.get_edge(twin).face as usize;

                let neighbor_normal = mesh.transform_normal(&mesh.normal(neighbor));
                let angle_diff = normal.angle(&neighbor_normal);

                if angle_diff > 0.1
                    && seen.insert((
                        edge.origin_vertex.min(edge.vertex),
                        edge.origin_vertex.max(edge.vertex),
                    ))
                {
                    cluster.mark_adjacency(edge.origin_vertex, edge.vertex);
                }
            }
        }

        println!("found {} edge runs", cluster.cluster_count());
        cluster
            .clusters()
            .map(|x| x.1.iter().copied().collect::<Vec<_>>())
            .collect()
    }

    pub fn detect_face_overhangs(&self, mesh: &Mesh) -> Vec<Vector3<f32>> {
        let mut out = Vec::new();
        let overhangs = self.overhanging_faces(mesh);

        let bed_size = self.bed_size.map(|x| x.raw());
        let x_count = (bed_size.x / self.config.face_support_spacing) as i32;
        let y_count = (bed_size.y / self.config.face_support_spacing) as i32;

        for x in 0..x_count {
            for y in 0..y_count {
                let pos = Vector2::new(
                    x as f32 * self.config.face_support_spacing - bed_size.x / 2.0,
                    y as f32 * self.config.face_support_spacing - bed_size.y / 2.0,
                );

                let mut intersections = Vec::new();
                for (idx, _angle) in overhangs.iter() {
                    let ray = Ray {
                        origin: mesh.inv_transform(&pos.to_homogeneous()),
                        direction: mesh.inv_transform_normal(&Vector3::z()),
                    };
                    let face = (mesh.face_verts_raw(*idx), mesh.normal(*idx));
                    if let Some(mut intersection) =
                        triangle_intersection::<primitive::Ray>(face, ray)
                    {
                        intersection.1 = mesh.transform(&intersection.1);
                        intersections.push(intersection.1);
                    }
                }

                intersections.sort_by_key(|x| OrderedFloat(x.z));
                intersections.dedup_by(|a, b| (a.z - b.z).abs() < 0.1);
                out.extend(intersections);
            }
        }

        out
    }
}
