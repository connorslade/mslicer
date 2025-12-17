use nalgebra::Vector3;

use crate::mesh::Mesh;

pub struct BoundingBox {
    min: Vector3<f32>,
    max: Vector3<f32>,
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: Vector3::repeat(f32::INFINITY),
            max: Vector3::repeat(f32::NEG_INFINITY),
        }
    }

    pub fn center(&self) -> Vector3<f32> {
        (self.min + self.max) / 2.0
    }

    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }

    pub fn longest_axis(&self) -> usize {
        let lengths = (self.max - self.min).abs();

        if lengths.x > lengths.y && lengths.x > lengths.z {
            return 0; // X
        }

        if lengths.y > lengths.x {
            return 1; // Y
        }

        2 // Z
    }

    pub fn expand(&mut self, point: Vector3<f32>) {
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
        let verts = mesh.vertices();
        let face = mesh.face(face_idx);

        self.expand(verts[face[0] as usize]);
        self.expand(verts[face[1] as usize]);
        self.expand(verts[face[2] as usize]);
    }

    // From https://iquilezles.org/articles/intersectors
    pub fn intersect_ray(&self, origin: Vector3<f32>, dir: Vector3<f32>) -> bool {
        let origin = origin - self.center();
        let inv_dir = dir.map(|x| x.recip());

        let n = inv_dir.component_mul(&origin);
        let k = inv_dir.abs().component_mul(&self.size());
        let tn = (-n - k).max();
        let tf = (-n + k).min();

        !(tn > tf || tf < 0.0)
    }
}
