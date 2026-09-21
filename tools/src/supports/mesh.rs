use std::{collections::HashMap, f32::consts::PI, range::Range, time::Instant};

use common::{geometry::convex_hull, units::Milimeters};
use nalgebra::{Vector2, Vector3};
use slicer::{builder::MeshBuilder, mesh::Mesh};
use tracing::info;

use crate::supports::{Support, SupportId};

pub type FaceMap = HashMap<SupportId, Range<u32>>;

#[derive(Clone)]
pub struct SupportPreset {
    pub support_radius: Milimeters,
    pub tip_radius: Milimeters,
    pub tip_length: Milimeters,
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub _rotation: f32,
}

pub fn build_supports<'a>(
    supports: impl Iterator<Item = &'a Support>,
    transform: Transform,
    resolution: u32,
) -> Option<(Mesh, FaceMap)> {
    let start = Instant::now();
    let mut builder = MeshBuilder::new();
    let mut raft_points = Vec::new();
    let mut map = HashMap::new();

    for support in supports {
        let (r, p) = (support.radius, resolution);
        let mut points = support.points;

        //rotate!?

        // ↓ incorrect!
        points[0] = points[0].component_mul(&transform.scale);
        points[1] = points[1].component_mul(&transform.scale);
        points[2] = points[2].component_mul(&transform.scale);

        points[0].z += transform.position.z;
        points[1].z += transform.position.z;
        points[2].z += transform.position.z;

        let start = builder.next_face_idx();
        builder.add_cylinder((points[0], points[1]), (support.tip_radius, r), p);
        builder.add_cylinder((points[1], points[2]), (r, r), p);
        builder.add_cylinder((points[2], points[2].xy().push(0.0)), (r, r), p);

        for i in 0..(p * 2) {
            let angle = i as f32 / p as f32 * PI;
            let normal = Vector2::new(angle.cos(), angle.sin());
            raft_points.push(points[2].xy() + normal * r);
        }

        builder.add_sphere(points[0], 0.2, p);
        builder.add_sphere(points[1], r, p);
        builder.add_sphere(points[2], r, p);

        let end = builder.next_face_idx();
        map.insert(support.id, Range::from(start..end));
    }

    build_raft(1.0, 1.0, &raft_points, &mut builder);
    (!builder.is_empty()).then(|| {
        let mesh = builder.build();
        let (faces, duration) = (mesh.face_count(), start.elapsed());
        info!("Generated support mesh with {faces} faces in {duration:?}");
        (mesh, map)
    })
}

// todo: the config values are stored in support generator struct but not used
// through it...
pub fn build_raft(
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

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            scale: Vector3::repeat(1.0),
            _rotation: 0.0,
        }
    }
}

impl Default for SupportPreset {
    fn default() -> Self {
        Self {
            support_radius: Milimeters::new(1.0),
            tip_radius: Milimeters::new(0.2),
            tip_length: Milimeters::new(3.0),
        }
    }
}
