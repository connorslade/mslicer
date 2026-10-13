use std::sync::Arc;

use nalgebra::{Matrix4, Vector3};
use serde::{Deserialize, Serialize};

use crate::geometry::{
    Primitive, Ray,
    mesh::{Mesh, MeshInner},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bounds3D {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Bounds3D {
    pub fn empty() -> Self {
        Self {
            min: Vector3::repeat(f32::INFINITY),
            max: Vector3::repeat(f32::NEG_INFINITY),
        }
    }

    pub fn from_verts(vertices: &[Vector3<f32>]) -> Self {
        let (min, max) = vertices.iter().fold(
            (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN)),
            |(min, max), v| (min.zip_map(v, f32::min), max.zip_map(v, f32::max)),
        );
        Self { min, max }
    }

    pub fn from_verts_transformed(vertices: &[Vector3<f32>], transform: &Matrix4<f32>) -> Self {
        let (min, max) = (vertices.iter().map(|v| (transform * v.push(1.0)).xyz())).fold(
            (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN)),
            |(min, max), v| (min.zip_map(&v, f32::min), max.zip_map(&v, f32::max)),
        );
        Self { min, max }
    }

    pub fn from_mesh(mesh: &Mesh) -> Self {
        Self::from_verts_transformed(mesh.vertices(), mesh.transformation_matrix())
    }

    pub fn extend(&mut self, other: &Bounds3D) {
        self.min = self.min.zip_map(&other.min, f32::min);
        self.max = self.max.zip_map(&other.max, f32::max);
    }

    pub fn expand_point(&mut self, point: &Vector3<f32>) {
        self.min = self.min.zip_map(point, f32::min);
        self.max = self.max.zip_map(point, f32::max);
    }

    pub fn expand_face(&mut self, mesh: &Arc<MeshInner>, face_idx: usize) {
        let face = mesh.faces[face_idx];
        self.expand_point(&mesh.vertices[face[0] as usize]);
        self.expand_point(&mesh.vertices[face[1] as usize]);
        self.expand_point(&mesh.vertices[face[2] as usize]);
    }

    pub fn is_empty(&self) -> bool {
        self.min > self.max
    }

    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }

    pub fn center(&self) -> Vector3<f32> {
        (self.max + self.min) / 2.0
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

    pub fn bounding_sphere(&self) -> (Vector3<f32>, f32) {
        let radius = (self.max.x - self.min.x).max(self.max.y - self.min.y) / 2.0;
        (self.center(), radius)
    }

    // Returns first intersection point. (Closer to ray origin)
    pub fn intersect<Type: Primitive>(&self, ray: Ray) -> Option<f32> {
        let (mut t_min, mut t_max) = (Type::min_t(), Type::max_t());

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

        (t_max >= Type::min_t()).then_some(t_min)
    }

    // Returns the squared distance from some point to the closest point on the AABB.
    // "Distance of Point to AABB" from Real-Time Collision Detection by Christer Ericson
    pub fn distance(&self, point: Vector3<f32>) -> f32 {
        let mut dist_sq = 0.0;
        for i in 0..3 {
            let v = point[i];
            (v < self.min[i]).then(|| dist_sq += (self.min[i] - v) * (self.min[i] - v));
            (v > self.max[i]).then(|| dist_sq += (v - self.max[i]) * (v - self.max[i]));
        }

        dist_sq
    }
}
