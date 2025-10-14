use nalgebra::Vector3;

use crate::mesh::Mesh;

pub mod segments_1d;
pub use segments_1d::Segments1D;

/// Intersects a plane with a triangle.
pub fn intersect_triangle(
    mesh: &Mesh,
    points: &[Vector3<f32>],
    face: usize,
    height: f32,
) -> Option<[Vector3<f32>; 2]> {
    // Get all the vertices of the face
    let face = mesh.face(face);
    let v0 = points[face[0] as usize];
    let v1 = points[face[1] as usize];
    let v2 = points[face[2] as usize];

    // By subtracting the height from each vertex z coord, we can now check if
    // each line segment is crossing the plane if one end is above and one is
    // below. We can use xor to do this quickly.
    let (a, b, c) = (v0.z - height, v1.z - height, v2.z - height);
    let (a_pos, b_pos, c_pos) = (a > 0.0, b > 0.0, c > 0.0);

    let mut out = [Vector3::zeros(); 2];
    let mut n = 0;

    // Closure called when the line segment from v0 to v1 is intersecting the
    // plane. t is how far along the line the intersection is and intersections,
    // it well the point that is intersecting with the plane.
    let mut push_intersection = |a: f32, b: f32, v0: Vector3<f32>, v1: Vector3<f32>| {
        let t = a / (a - b);
        let intersection = v0 + t * (v1 - v0);
        out[n] = intersection;
        n += 1;
    };

    // And as you can see my aversion to else blocks now includes if blocks...
    // Anyway here we just check each line segment of the face is intersecting,
    // if it is we push the intersection to the out vec.
    (a_pos ^ b_pos).then(|| push_intersection(a, b, v0, v1));
    (b_pos ^ c_pos).then(|| push_intersection(b, c, v1, v2));
    (c_pos ^ a_pos).then(|| push_intersection(c, a, v2, v0));

    (n == 2).then_some(out)
}
