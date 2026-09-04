// todo: not sure supports should be classified as a tool but fuck who cares
// anyway noting in life really matters like its all about living your best life
// and i shouldn't let something so small as a rust module being located in a
// crate whose name doesn't completely convey the importance of the feature to
// the overall application to which it belongs even slightly lower appreciation
// for the world. one life is all we got you know, unless you're spiritual or
// wtvr but you have to make it count, do important things, don't waste time on
// the little things that don't deserve you mind.

use common::{geometry::convex_hull, units::Milimeters};
use nalgebra::Vector2;
use nalgebra::Vector3;
use slicer::{builder::MeshBuilder, geometry::bvh::Bvh, half_edge::HalfEdgeMesh, mesh::Mesh};
use tracing::info;

pub mod detect;
pub mod quazirandom;

pub struct SupportGenerator<'a> {
    config: &'a SupportConfig,
    bed_size: Vector3<Milimeters>,
}

pub struct Support {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub radius: f32,
}

pub struct SupportConfig {
    /// Support generation
    pub support_radius: f32,
    pub tip_radius: f32,
    pub tip_length: f32,
    pub precision: u32,

    pub min_spacing: f32,

    pub raft_height: f32,
    pub raft_offset: f32,

    pub max_angle: f32,
    pub face_support_spacing: f32,
    pub edge_support_spacing: f32,
    pub edge_angle_delta: f32,
}

pub struct SupportPlacement {
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
}

impl<'a> SupportGenerator<'a> {
    pub fn new(config: &'a SupportConfig, bed_size: Vector3<Milimeters>) -> Self {
        Self { config, bed_size }
    }

    pub fn generate_supports(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
        bvh: &Bvh,
    ) -> Vec<[Vector3<f32>; 3]> {
        let mut overhangs = Vec::new();
        let min_dist = self.config.min_spacing;

        let overhanging_faces = self.overhanging_faces(mesh);
        let mut faces = self.place_face_supports(mesh, &overhanging_faces);
        let mut edges = self.place_edge_supports(mesh, half_edge, &overhanging_faces);
        for overhang in edges.iter() {
            // i know its n²... shut up.
            faces.retain(|x| (x.point - overhang.point).magnitude() > min_dist);
        }

        let points = self.place_point_supports(mesh, half_edge);
        for overhang in points.iter() {
            faces.retain(|x| (x.point - overhang.point).magnitude() > min_dist);
            edges.retain(|x| (x.point - overhang.point).magnitude() > min_dist);
        }

        info!(
            "Placed {} supports. {{ point: {}, face: {}, edge: {} }}",
            points.len() + faces.len() + edges.len(),
            points.len(),
            faces.len(),
            edges.len()
        );
        overhangs.extend([points, faces, edges].into_iter().flatten());

        overhangs
            .into_iter()
            .filter_map(|x| {
                let tip_start = x.point + x.normal * self.config.tip_length;
                let mid = route_support(mesh, bvh, tip_start);
                mid.map(|mid| [x.point, tip_start, mid])
            })
            .collect()
    }

    // let mut builder = MeshBuilder::new();
    // let raft_points = self.build_support_mesh(mesh, bvh, &overhangs, &mut builder);
    // self.build_raft_mesh(&raft_points, &mut builder);
}

/// Returns the middle of the three points defining a support. The final point
/// (that touches the build plate) is just this returned point projected down.
pub fn route_support(mesh: &Mesh, bvh: &Bvh, position: Vector3<f32>) -> Option<Vector3<f32>> {
    let mut point = position;
    let mut momentum = Vector3::zeros();
    let beta = 0.9;

    for _ in 0..100 {
        let closest = bvh.closest(mesh, point).unwrap();
        let grad = point - mesh.transform(&closest.position);

        momentum = beta * momentum + (1.0 - beta) * grad;
        point += momentum.xy().push(momentum.z.min(0.0)).normalize() * closest.t.min(1.0);

        if bvh.intersect_ray(mesh, point, -Vector3::z()).is_none() {
            return Some(point);
        }
    }

    None
}

// todo: the config values are stored in support generator struct but not used
// through it...
pub fn build_raft_mesh(
    raft_offset: f32,
    raft_height: f32,
    points: &[Vector2<f32>],
    builder: &mut MeshBuilder,
) {
    let hull = convex_hull(points);
    let idx = builder.next_idx();
    for i in 0..hull.len() {
        let point = hull[i];
        let next = hull[(i + 1) % hull.len()];
        let prev = hull[(i + hull.len() - 1) % hull.len()];

        let edge_1 = next - point;
        let edge_2 = point - prev;
        let offset = (Vector2::new(edge_1.y, -edge_1.x).normalize()
            + Vector2::new(edge_2.y, -edge_2.x).normalize())
        .normalize();

        builder.add_vertex(point.push(0.0));
        builder.add_vertex((point + offset * raft_offset).push(raft_height));
    }

    let verts = builder.next_idx() - idx;
    for i in (0..verts).step_by(2) {
        if i != 0 && i + 3 < verts {
            builder.add_face([idx, idx + i + 2, idx + i]);
            builder.add_face([idx + 1, idx + i + 1, idx + i + 3]);
        }

        builder.add_quad([
            idx + i % verts,
            idx + (i + 1) % verts,
            idx + (i + 2) % verts,
            idx + (i + 3) % verts,
        ]);
    }
}

impl Default for SupportConfig {
    fn default() -> Self {
        Self {
            support_radius: 1.0,
            tip_radius: 0.2,
            tip_length: 3.0,
            raft_height: 1.0,
            raft_offset: 1.0,
            min_spacing: 5.0,
            precision: 10,
            max_angle: 30.0,
            face_support_spacing: 50.0,
            edge_angle_delta: 0.1,
            edge_support_spacing: 20.0,
        }
    }
}
