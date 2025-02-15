use nalgebra::Vector3;

use crate::mesh::Mesh;

/// Acceleration structure for mesh slicing. By spiting the mesh into segments
/// along the slicing axis and adding references to all the triangles that
/// overlap each segment, to slice a layer, you don't need to loop through every
/// triangle in the mesh to find all intersecting faces.
pub struct Segments1D {
    start_height: f32,
    layer_height: f32,

    layers: Vec<Vec<usize>>,
    transformed_points: Vec<Vector3<f32>>,
}

impl Segments1D {
    /// Creates a new Segments structure from a given mesh and segment count.
    pub fn from_mesh(mesh: &Mesh, layer_count: usize) -> Self {
        let (min, max) = mesh.bounds();

        // Caching transformed points makes slicing faster.
        let transformed_points = mesh
            .vertices()
            .iter()
            .map(|x| mesh.transform(x))
            .collect::<Vec<_>>();

        // Create a bin for each layer
        let layer_height = (max.z - min.z) / layer_count as f32;
        let mut layers = vec![Vec::new(); layer_count + 1];

        // Adds the index of each face into all of the segments it covers.
        for face in 0..mesh.face_count() {
            let (min_height, max_height) = triangle_bounds(mesh, &transformed_points, face);
            let (min_layer, max_layer) = (
                ((min_height - min.z) / layer_height) as usize,
                ((max_height - min.z) / layer_height).round() as usize,
            );

            for layer in layers.iter_mut().take(max_layer + 1).skip(min_layer) {
                layer.push(face);
            }
        }

        Self {
            start_height: min.z,
            layer_height,

            layers,
            transformed_points,
        }
    }

    /// Intersects a plane with the mesh this Segments instance was built with.
    /// Returns a list of line segments along with the direction of the face (left or right).
    pub fn intersect_plane(&self, mesh: &Mesh, height: f32) -> Vec<([Vector3<f32>; 2], bool)> {
        let mut out = Vec::new();

        let layer = (height - self.start_height) / self.layer_height;
        if layer < 0.0 || layer >= self.layers.len() as f32 {
            return out;
        }

        for &face in self.layers[layer as usize].iter() {
            let segment = intersect_triangle(mesh, &self.transformed_points, face, height);
            if let Some(segment) = segment {
                out.push((segment, mesh.transform_normal(mesh.normal(face)).x > 0.0));
            }
        }

        out
    }
}

/// Gets the min and max heights of the vertices of a face.
fn triangle_bounds(mesh: &Mesh, points: &[Vector3<f32>], triangle: usize) -> (f32, f32) {
    let triangle = mesh.faces()[triangle];
    let heights = (
        points[triangle[0] as usize].z,
        points[triangle[1] as usize].z,
        points[triangle[2] as usize].z,
    );

    (
        heights.0.min(heights.1).min(heights.2),
        heights.0.max(heights.1).max(heights.2),
    )
}

/// Intersects a plane with a triangle.
fn intersect_triangle(
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
