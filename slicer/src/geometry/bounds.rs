use std::{iter::Sum, sync::Arc};

use nalgebra::{Matrix4, Vector2, Vector3};
use serde::{Deserialize, Serialize};

use crate::geometry::{Primitive, Ray, mesh::MeshInner};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bounds3D {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

#[derive(Clone, Copy)]
pub struct Bounds2D<T = f32> {
    pub min: Vector2<T>,
    pub max: Vector2<T>,
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

impl<T: Copy> Bounds2D<T> {
    pub fn new_point(point: Vector2<T>) -> Self {
        Bounds2D {
            min: point,
            max: point,
        }
    }
}

impl Bounds2D<f32> {
    pub const EMPTY: Self = Self {
        min: Vector2::new(f32::MAX, f32::MAX),
        max: Vector2::new(f32::MIN, f32::MIN),
    };

    pub fn new_containing(points: &[Vector2<f32>]) -> Self {
        (points.iter()).fold(Self::EMPTY, |a, v| a.include_bound(Self::new_point(*v)))
    }

    pub fn include_bound(self, other: Self) -> Self {
        Self {
            min: self.min.zip_map(&other.min, f32::min),
            max: self.max.zip_map(&other.max, f32::max),
        }
    }

    pub fn include_bound_mut(&mut self, other: Self) {
        self.min = self.min.zip_map(&other.min, f32::min);
        self.max = self.max.zip_map(&other.max, f32::max);
    }

    pub fn offset(self, offset: Vector2<f32>) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    pub fn size(&self) -> Vector2<f32> {
        self.max - self.min
    }

    pub fn contains(&self, point: Vector2<f32>) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

impl Bounds2D<f64> {
    pub const EMPTY: Self = Self {
        min: Vector2::new(f64::MAX, f64::MAX),
        max: Vector2::new(f64::MIN, f64::MIN),
    };

    pub fn new_containing(points: &[Vector2<f64>]) -> Self {
        (points.iter()).fold(Self::EMPTY, |a, v| a.include_bound(Self::new_point(*v)))
    }

    pub fn include_bound(self, other: Self) -> Self {
        Self {
            min: self.min.zip_map(&other.min, f64::min),
            max: self.max.zip_map(&other.max, f64::max),
        }
    }

    pub fn include_bound_mut(&mut self, other: Self) {
        self.min = self.min.zip_map(&other.min, f64::min);
        self.max = self.max.zip_map(&other.max, f64::max);
    }

    pub fn offset(self, offset: Vector2<f64>) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    pub fn size(&self) -> Vector2<f64> {
        self.max - self.min
    }
}

impl Sum for Bounds2D<f32> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Self::EMPTY;
        iter.for_each(|bound| acc.include_bound_mut(bound));
        acc
    }
}

impl Sum for Bounds2D<f64> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Self::EMPTY;
        iter.for_each(|bound| acc.include_bound_mut(bound));
        acc
    }
}
