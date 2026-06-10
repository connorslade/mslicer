use std::{iter::Sum, ops::Add};

use nalgebra::Vector2;

#[derive(Clone, Copy)]
pub struct Bounds2D {
    pub min: Vector2<f32>,
    pub max: Vector2<f32>,
}

impl Bounds2D {
    pub const EMPTY: Self = Self {
        min: Vector2::new(0.0, 0.0),
        max: Vector2::new(0.0, 0.0),
    };

    pub fn new_containing(points: &[Vector2<f32>]) -> Self {
        points.iter().fold(Self::EMPTY, |a, v| a + *v)
    }

    pub fn size(&self) -> Vector2<f32> {
        self.max - self.min
    }
}

impl Add<Bounds2D> for Bounds2D {
    type Output = Bounds2D;

    fn add(self, rhs: Bounds2D) -> Self::Output {
        Self {
            min: self.min.zip_map(&rhs.min, f32::min),
            max: self.max.zip_map(&rhs.max, f32::max),
        }
    }
}

impl Add<Vector2<f32>> for Bounds2D {
    type Output = Bounds2D;

    fn add(self, rhs: Vector2<f32>) -> Self::Output {
        Self {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl Sum for Bounds2D {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Self::EMPTY;
        for bound in iter {
            acc = acc + bound;
        }

        acc
    }
}
