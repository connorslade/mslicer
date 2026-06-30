use nalgebra::{Vector2, Vector3};

use crate::misc::bounds::Bounds2D;

mod annealing;
mod cache;
mod nfp;
pub use self::{
    annealing::{AutoLayoutAnnealing, Rotation},
    cache::{CacheEntry, Hull, LayoutCache},
    nfp::AutoLayoutNfp,
};

#[derive(Clone)]
pub struct Model {
    model: u32,
    mesh: usize,

    position: Vector2<f32>,
    rotation: f32,
}

#[derive(Clone)]
pub struct Placement {
    pub model: u32,
    pub position: Vector3<f32>,
    pub rotation: f32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Objective {
    Area,
    Perimeter,
    LongestAxis,
}

impl Model {
    pub fn new(model: u32, mesh: usize) -> Self {
        Self {
            model,
            mesh,

            position: Vector2::zeros(),
            rotation: 0.0,
        }
    }

    pub fn entry(&self) -> CacheEntry {
        CacheEntry::new(self.mesh, self.rotation)
    }
}

impl Objective {
    pub const ALL: [Self; 3] = [Self::Area, Self::Perimeter, Self::LongestAxis];

    pub fn name(&self) -> &str {
        match self {
            Self::Area => "Area",
            Self::Perimeter => "Perimeter",
            Self::LongestAxis => "Longest Axis",
        }
    }

    pub fn eval(&self, platform: Vector2<f32>, bounds_penalty: f32, bounds: Bounds2D) -> f32 {
        let size = bounds.size();
        let mut score = match self {
            Self::Area => size.x * size.y,
            Self::Perimeter => size.x + size.y,
            Self::LongestAxis => size.x.max(size.y),
        };

        let (x, y) = (size.x > platform.x, size.y > platform.y);
        if x && y {
            score += bounds_penalty * (size.x * size.y - platform.x * platform.y);
        } else if x {
            score += bounds_penalty * (size.x * size.y - platform.x * size.y);
        } else if y {
            score += bounds_penalty * (size.x * size.y - size.x * platform.y);
        }

        score
    }
}
