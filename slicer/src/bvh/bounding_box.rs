use nalgebra::Vector3;

use crate::mesh::Mesh;

pub struct BoundingBox {
    min: Vector3<f32>,
    max: Vector3<f32>,
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: Vector3::zeros(),
            max: Vector3::zeros(),
        }
    }

    pub fn center(&self) -> Vector3<f32> {
        (self.min + self.max) / 2.0
    }

    pub fn longest_axis(&self) -> usize {
        let lengths = (self.max - self.min).abs();

        if lengths.x > lengths.y && lengths.x > lengths.z {
            return 0;
        }

        if lengths.y > lengths.x {
            return 1;
        }

        2
    }

    pub fn expand_point(&mut self, point: Vector3<f32>) {
        self.min = Vector3::new(
            self.min.x.min(point.x),
            self.min.y.min(point.y),
            self.min.z.min(point.z),
        );
        self.max = Vector3::new(
            self.max.x.max(point.x),
            self.max.y.max(point.y),
            self.max.z.max(point.z),
        );
    }

    pub fn expand_face(&mut self, mesh: &Mesh, face_idx: usize) {
        let face = mesh.faces[face_idx];
        self.expand_point(mesh.vertices[face[0] as usize]);
        self.expand_point(mesh.vertices[face[1] as usize]);
        self.expand_point(mesh.vertices[face[2] as usize]);
    }

    // todo: is is more effecent to precalculate pos - normal <dot> or not
    pub fn intersect_plane(&self, pos: Vector3<f32>, normal: Vector3<f32>) -> bool {
        let (min, max) = (self.min, self.max);
        let cube_vertices = [
            // Bottom vertices
            Vector3::new(min.x, min.y, min.z),
            Vector3::new(max.x, min.y, min.z),
            Vector3::new(min.x, max.y, min.z),
            Vector3::new(max.x, max.y, min.z),
            // Top vertices
            Vector3::new(min.x, min.y, max.z),
            Vector3::new(max.x, min.y, max.z),
            Vector3::new(min.x, max.y, max.z),
            Vector3::new(max.x, max.y, max.z),
        ];

        let intersection_test = |a: usize, b: usize| {
            let a = (cube_vertices[a] - pos).dot(&normal);
            let b = (cube_vertices[b] - pos).dot(&normal);
            (a > 0.0) ^ (b > 0.0)
        };

        // (0..4).any(|x| intersection_test(x, (x + 1) % 4))
        //     || (0..4).any(|x| intersection_test(x + 4, (x + 1) % 4 + 4))
        //     || (0..4).any(|x| intersection_test(x, x + 4))

        (0..4).any(|x| intersection_test(x, x + 4))
    }
}
