use std::f32::consts::PI;

use common::{geometry::convex_hull, units::Milimeters};
use nalgebra::Vector2;
use nalgebra::Vector3;
use slicer::{builder::MeshBuilder, geometry::bvh::Bvh, half_edge::HalfEdgeMesh, mesh::Mesh};

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

    pub raft_height: f32,
    pub raft_offset: f32,

    pub precision: u32,

    /// Overhang detection
    pub max_angle: f32,
    pub face_support_spacing: f32,
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
        debug: &mut Vec<[Vector3<f32>; 2]>,
    ) -> Option<Mesh> {
        let mut overhangs = Vec::new();

        let faces = self.overhanging_faces(mesh);
        overhangs.extend(self.place_point_supports(mesh, half_edge).iter());
        overhangs.extend(self.place_face_supports(mesh, &faces).iter());
        overhangs.extend(self.place_edge_supports(mesh, half_edge, &faces).iter());

        let mut builder = MeshBuilder::new();
        let raft_points = self.build_support_mesh(mesh, bvh, &overhangs, &mut builder);
        self.build_raft_mesh(&raft_points, &mut builder);

        (!builder.is_empty()).then(|| builder.build())
    }

    fn build_support_mesh(
        &self,
        mesh: &Mesh,
        bvh: &Bvh,
        overhangs: &[Vector3<f32>],
        builder: &mut MeshBuilder,
    ) -> Vec<Vector2<f32>> {
        let (r, tr, p) = (
            self.config.support_radius,
            self.config.tip_radius,
            self.config.precision,
        );

        let mut raft_points = Vec::new();
        for &point in overhangs {
            let start = point - Vector3::z();
            if let Some(lines) = route_support(mesh, bvh, start) {
                builder.add_cylinder((point, start), (tr, r), p);
                builder.add_cylinder((lines[0], lines[1]), (r, r), p);
                builder.add_cylinder((lines[1], lines[2]), (r, r), p);

                for i in 0..(p * 2) {
                    let angle = i as f32 / p as f32 * PI;
                    let normal = Vector2::new(angle.cos(), angle.sin());
                    raft_points.push(lines[2].xy() + normal * r);
                }

                builder.add_sphere(point, 0.2, p);
                builder.add_sphere(lines[0], r, p);
                builder.add_sphere(lines[1], r, p);
            }
        }

        raft_points
    }

    fn build_raft_mesh(&self, points: &[Vector2<f32>], builder: &mut MeshBuilder) {
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
            builder.add_vertex(
                (point + offset * self.config.raft_offset).push(self.config.raft_height),
            );
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
}

pub fn route_support(mesh: &Mesh, bvh: &Bvh, position: Vector3<f32>) -> Option<[Vector3<f32>; 3]> {
    let mut point = position;
    let mut momentum = Vector3::zeros();
    let beta = 0.9;

    for _ in 0..100 {
        let closest = bvh.closest(mesh, point).unwrap();
        let grad = point - mesh.transform(&closest.position);

        momentum = beta * momentum + (1.0 - beta) * grad;
        point += momentum.xy().push(momentum.z.min(0.0)).normalize() * closest.t.min(1.0);

        if bvh.intersect_ray(mesh, point, -Vector3::z()).is_none() {
            return Some([position, point, point.xy().to_homogeneous()]);
        }
    }

    None
}

impl Default for SupportConfig {
    fn default() -> Self {
        Self {
            support_radius: 1.0,
            tip_radius: 0.2,
            raft_height: 1.0,
            raft_offset: 1.0,
            precision: 10,
            max_angle: 30.0,
            face_support_spacing: 5.0,
        }
    }
}
