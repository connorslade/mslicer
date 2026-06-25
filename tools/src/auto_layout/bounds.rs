use std::iter::Sum;

use nalgebra::Vector2;

#[derive(Clone, Copy)]
pub struct Bounds2D {
    pub min: Vector2<f32>,
    pub max: Vector2<f32>,
}

impl Bounds2D {
    pub const EMPTY: Self = Self {
        min: Vector2::new(f32::MAX, f32::MAX),
        max: Vector2::new(f32::MIN, f32::MIN),
    };

    pub fn new_point(point: Vector2<f32>) -> Self {
        Bounds2D {
            min: point,
            max: point,
        }
    }

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
}

impl Sum for Bounds2D {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Self::EMPTY;
        iter.for_each(|bound| acc.include_bound_mut(bound));
        acc
    }
}
