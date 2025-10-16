use nalgebra::Vector3;

use crate::{intersection::triangle::plane_triangle_intersection, mesh::Mesh};

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
            let segment = plane_triangle_intersection(mesh, &self.transformed_points, face, height);
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
