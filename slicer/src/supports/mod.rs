use nalgebra::Vector3;

use crate::{geometry::bvh::Bvh, mesh::Mesh};

pub mod line;
pub mod overhangs;

pub fn route_support(mesh: &Mesh, bvh: &Bvh, position: Vector3<f32>) -> Option<[Vector3<f32>; 3]> {
    let mut point = position;
    for _ in 0..50 {
        let (distance, grad) = grad(bvh, mesh, point, 0.1);
        point += grad.xy().normalize().to_homogeneous() * distance.min(1.0);
        if bvh.intersect_ray(mesh, point, -Vector3::z()).is_none() {
            return Some([position, point, point.xy().to_homogeneous()]);
        }
    }

    None
}

fn grad(bvh: &Bvh, mesh: &Mesh, point: Vector3<f32>, delta: f32) -> (f32, Vector3<f32>) {
    let sdf = |point| bvh.closest(mesh, point).unwrap().t;

    let distance = sdf(point);
    let dx = sdf(point + Vector3::x() * delta);
    let dy = sdf(point + Vector3::y() * delta);
    let dz = sdf(point + Vector3::z() * delta);
    let grad = (Vector3::new(dx, dy, dz) - Vector3::repeat(distance)) / delta;

    (distance, grad)
}
