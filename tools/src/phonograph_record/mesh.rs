use std::f32::consts::PI;

use nalgebra::{Vector2, Vector3};
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

/// Triangulates a the polygon is in the XY plane.
///
/// Reference: https://nils-olovsson.se/articles/ear_clipping_triangulation
pub fn ear_clipping(polygon: &[u32], positions: &[Vector3<f32>], out: &mut Vec<[u32; 3]>) {
    let n = polygon.len();
    let mut prev = (0..n).map(|i| (i + n - 1) % n).collect::<Vec<_>>();
    let mut next = (0..n).map(|i| (i + 1) % n).collect::<Vec<_>>();

    let mut i = 0;
    let mut remaining = n;

    while remaining > 3 {
        if remaining % 100 == 0 {
            println!("{:.2}%", (1.0 - remaining as f32 / n as f32) * 100.0);
        }

        let vi @ [idx_a, _, idx_c] = [prev[i], i, next[i]];
        let [a, b, c] = vi.map(|i| positions[polygon[i] as usize].xy());

        // check if this triangle is convex and doesn't contain any other
        // points. if so, it's an ear!
        let convex = (a - b).perp(&(c - b)) >= 0.0;
        let mut j = idx_c;
        let empty = (0..remaining - 3).all(|_| {
            j = next[j];
            let point = positions[polygon[j] as usize].xy();
            !point_in_triangle([a, b, c], point)
        });

        if convex && empty {
            out.push(vi.map(|x| x as u32));

            next[idx_a] = idx_c;
            prev[idx_c] = idx_a;
            remaining -= 1;
            i = idx_a;
        } else {
            i = next[i];
        }
    }

    out.push([polygon[prev[i]], polygon[i], polygon[next[i]]]);
}

// fn point_in_remaining()

fn point_in_triangle([a, b, c]: [Vector2<f32>; 3], p: Vector2<f32>) -> bool {
    let d0 = (a - b).perp(&(p - b));
    let d1 = (b - c).perp(&(p - c));
    let d2 = (c - a).perp(&(p - a));
    (d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0) || (d0 <= 0.0 && d1 <= 0.0 && d2 <= 0.0)
}
