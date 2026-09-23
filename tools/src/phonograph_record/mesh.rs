use std::f32::consts::PI;

use nalgebra::Vector3;
use slicer::builder::{MeshBuilder, orthogonal_basis};

pub fn add_cylinder_inner(
    builder: &mut MeshBuilder,
    bottom: Vector3<f32>,
    height: f32,
    radius: f32,
    precision: u32,
) {
    let (a, b) = (bottom, bottom + Vector3::z() * height);
    let [u, v] = orthogonal_basis((a - b).normalize());

    let mut first = None;
    let mut last = None;
    for i in 0..(precision * 2) {
        let angle = i as f32 / precision as f32 * PI;
        let normal = u * angle.sin() + v * angle.cos();

        let top = builder.add_vertex(b + normal * radius);
        let bottom = builder.add_vertex(a + normal * radius);

        if let Some((last_top, last_bottom)) = last {
            builder.add_quad_flipped([last_bottom, last_top, bottom, top]);
        }

        last = Some((top, bottom));
        first.is_none().then(|| first = last);
    }

    if let Some((last_top, last_bottom)) = last
        && let Some((first_top, first_bottom)) = first
    {
        builder.add_quad_flipped([last_bottom, last_top, first_bottom, first_top]);
    }
}
