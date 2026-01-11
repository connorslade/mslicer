use nalgebra::Vector3;

use crate::{geometry::bvh::Bvh, mesh::Mesh};

pub mod line;
pub mod overhangs;

const DELTA: f32 = 0.1;

pub fn route_support(mesh: &Mesh, bvh: &Bvh, position: Vector3<f32>) -> Option<[Vector3<f32>; 3]> {
    let sdf = |point| bvh.closest(mesh, point).unwrap().t;

    let mut point = position;
    for _ in 0..50 {
        let distance = sdf(point);
        let dx = sdf(point + Vector3::x() * DELTA);
        let dy = sdf(point + Vector3::y() * DELTA);
        let dz = sdf(point + Vector3::z() * DELTA);
        let grad = (Vector3::new(dx, dy, dz) - Vector3::repeat(distance)) / DELTA;

        point += grad.xy().normalize().to_homogeneous() * distance.min(1.0);
        if bvh.intersect_ray(mesh, point, -Vector3::z()).is_none() {
            return Some([position, point, point.xy().to_homogeneous()]);
        }
    }

    None
}
