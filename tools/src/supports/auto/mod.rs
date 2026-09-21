use common::units::{Milimeter, Milimeters};
use nalgebra::Vector3;
use slicer::{geometry::bvh::Bvh, half_edge::HalfEdgeMesh, mesh::Mesh};
use tracing::info;

use crate::supports::mesh::SupportPreset;

pub mod detect;
pub mod quazirandom;

pub struct AutoPlacement<'a> {
    pub placement: &'a PlacementConfig,
    pub support: &'a SupportPreset,

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

impl<'a> AutoPlacement<'a> {
    pub fn new(
        placement: &'a PlacementConfig,
        support: &'a SupportPreset,
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

        overhangs
            .into_iter()
            .filter_map(|x| {
                let tip_start = x.point + x.normal * self.support.tip_length.get::<Milimeter>();
                let mid = route_support(mesh, bvh, tip_start);
                mid.map(|mid| [x.point, tip_start, mid])
            })
            .collect()
    }
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
