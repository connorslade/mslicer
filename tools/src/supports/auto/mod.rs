use std::f32::consts::TAU;

use common::{
    progress::Progress,
    units::{Milimeter, Milimeters},
};
use nalgebra::Vector3;
use slicer::geometry::{
    mesh::{HalfEdgeMesh, Mesh, orthogonal_basis},
    partitioning::Bvh,
};
use tracing::info;

use crate::supports::mesh::SupportPreset;

pub mod detect;
pub mod quazirandom;

pub struct AutoPlacement {
    pub placement: PlacementConfig,
    pub support: SupportPreset,

    pub bed_size: Vector3<Milimeters>,
}

#[derive(Clone)]
pub struct PlacementConfig {
    pub min_spacing: f32,
    pub max_angle: f32,
    pub face_support_spacing: f32,
    pub edge_support_spacing: f32,
    pub edge_angle_delta: f32,
}

impl AutoPlacement {
    pub fn new(
        placement: PlacementConfig,
        support: SupportPreset,
        bed_size: Vector3<Milimeters>,
    ) -> Self {
        Self {
            placement,
            support,

            bed_size,
        }
    }

    pub fn generate_supports(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
        bvh: &Bvh,
        progress: &Progress,
    ) -> Vec<[Vector3<f32>; 3]> {
        let mut overhangs = Vec::new();
        let min_dist = self.placement.min_spacing;

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

        progress.set_total(overhangs.len() as u64);
        let radius = (self.support.radius + self.support.margin).get::<Milimeter>();
        overhangs
            .into_iter()
            .filter_map(|x| {
                let tip_start = x.point + x.normal * self.support.tip_length.get::<Milimeter>();
                let mid = route_support(mesh, bvh, radius, tip_start);
                progress.add_complete(1);
                mid.map(|mid| [x.point, tip_start, mid])
            })
            .collect()
    }
}

/// Returns the middle of the three points defining a support. The final point
/// (that touches the build plate) is just this returned point projected down.
pub fn route_support(
    mesh: &Mesh,
    bvh: &Bvh,
    radius: f32,
    position: Vector3<f32>,
) -> Option<Vector3<f32>> {
    let mut point = position;
    let mut momentum = Vector3::zeros();
    let beta = 0.9;

    // todo: try searching through angles instead of the gradient decent
    for _ in 0..100 {
        let closest = bvh.closest(mesh, point).unwrap();
        let grad = point - mesh.transform(&closest.position);

        momentum = beta * momentum + (1.0 - beta) * grad;
        point += momentum.xy().push(momentum.z.min(0.0)).normalize() * closest.t.min(1.0);

        if !intersect_cylinder(mesh, bvh, radius, (position, point))
            && !intersect_cylinder(mesh, bvh, radius, (point, point.xy().to_homogeneous()))
        {
            while point.z > 2.0 && !intersect_cylinder(mesh, bvh, radius, (position, point)) {
                point.z -= 1.0
            }

            return Some(point);
        }
    }

    None
}

fn intersect_cylinder(
    mesh: &Mesh,
    bvh: &Bvh,
    radius: f32,
    (start, end): (Vector3<f32>, Vector3<f32>),
) -> bool {
    let n = 4;

    let [u, v] = orthogonal_basis((end - start).normalize());
    for i in 0..n {
        let t = i as f32 / n as f32 * TAU;
        let offset = (u * t.cos() + v * t.sin()) * radius;

        if bvh
            .intersect_segment(mesh, start + offset, end + offset)
            .is_some()
        {
            return true;
        }
    }

    false
}

impl Default for PlacementConfig {
    fn default() -> Self {
        Self {
            min_spacing: 5.0,
            max_angle: 30.0,
            face_support_spacing: 50.0,
            edge_angle_delta: 0.1,
            edge_support_spacing: 20.0,
        }
    }
}
