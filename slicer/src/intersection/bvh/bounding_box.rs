use nalgebra::Vector3;

use crate::{intersection::bvh::Ray, mesh::Mesh};

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

    // Returns first intersection point. (Closer to ray origin)
    pub fn intersect<const SEGMENT: bool>(&self, ray: Ray) -> Option<f32> {
        let (mut t_min, mut t_max) = (0_f32, [f32::INFINITY, 1_f32][SEGMENT as usize]);

        for i in 0..3 {
            if ray.direction[i].abs() > 1e-8 {
                let t0 = (self.min[i] - ray.origin[i]) / ray.direction[i];
                let t1 = (self.max[i] - ray.origin[i]) / ray.direction[i];
                let (t0, t1) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
                t_min = t_min.max(t0);
                t_max = t_max.min(t1);
                if t_min > t_max {
                    return None;
                }
            } else if ray.origin[i] < self.min[i] || ray.origin[i] > self.max[i] {
                return None;
            }
        }

        (t_max >= 0.0).then_some(t_min)
    }
}
