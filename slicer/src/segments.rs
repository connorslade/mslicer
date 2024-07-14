use nalgebra::Vector3;

use crate::mesh::Mesh;

pub struct Segments {
    start_height: f32,
    layer_height: f32,

    layers: Vec<Vec<usize>>,
}

impl Segments {
    pub fn from_mesh(mesh: &Mesh, layer_count: usize) -> Self {
        let (min, max) = mesh.minmax_point();
        let transformed_points = mesh
            .vertices
            .iter()
            .map(|x| mesh.transform(x))
            .collect::<Vec<_>>();

        let layer_height = (max.z - min.z) / layer_count as f32;
        let mut layers = vec![Vec::new(); layer_count + 2];

        for face in 0..mesh.faces.len() {
            let (min_height, max_height) = minmax_triangle_height(mesh, &transformed_points, face);
            let (min_layer, max_layer) = (
                ((min_height - min.z) / layer_height) as usize,
                ((max_height - min.z) / layer_height) as usize,
            );

            for layer in layers
                .iter_mut()
                .take(max_layer + 2)
                .skip(min_layer.saturating_sub(1))
            {
                layer.push(face);
            }
        }

        Self {
            start_height: min.z,
            layer_height,
            layers,
        }
    }

    pub fn intersect_plane(&self, mesh: &Mesh, height: f32) -> Vec<Vector3<f32>> {
        let mut out = Vec::new();

        let layer = (height - self.start_height) / self.layer_height;
        for &face in self.layers[layer as usize].iter() {
            intersect_triangle(mesh, face, height, &mut out);
        }

        out
    }
}

fn minmax_triangle_height(mesh: &Mesh, points: &[Vector3<f32>], triangle: usize) -> (f32, f32) {
    let triangle = mesh.faces[triangle];
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

fn intersect_triangle(mesh: &Mesh, face: usize, height: f32, out: &mut Vec<Vector3<f32>>) {
    let face = mesh.faces[face];
    let v0 = mesh.vertices[face[0] as usize];
    let v1 = mesh.vertices[face[1] as usize];
    let v2 = mesh.vertices[face[2] as usize];

    let (a, b, c) = (v0.z - height, v1.z - height, v2.z - height);
    let (a_pos, b_pos, c_pos) = (a > 0.0, b > 0.0, c > 0.0);

    let mut push_intersection = |a: f32, b: f32, v0: Vector3<f32>, v1: Vector3<f32>| {
        let (v0, v1) = (mesh.transform(&v0), mesh.transform(&v1));
        let t = a / (a - b);
        let intersection = v0 + t * (v1 - v0);
        out.push(intersection);
    };

    (a_pos ^ b_pos).then(|| push_intersection(a, b, v0, v1));
    (b_pos ^ c_pos).then(|| push_intersection(b, c, v1, v2));
    (c_pos ^ a_pos).then(|| push_intersection(c, a, v2, v0));
}
