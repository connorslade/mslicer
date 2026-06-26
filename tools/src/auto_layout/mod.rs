use std::iter;

use itertools::Itertools;
use nalgebra::{Rotation2, Vector2, Vector3};

use crate::auto_layout::bounds::Bounds2D;

mod annealing;
mod bounds;
mod nfp;
pub use self::{
    annealing::{AutoLayoutAnnealing, Rotation},
    nfp::AutoLayoutNFP,
};

#[derive(Clone)]
pub struct Model {
    id: u32,
    origin: Vector3<f32>,
    base_rotation: f32,

    hull: Vec<Vector2<f32>>,
    bounds: Bounds2D,

    offset: Vector2<f32>,
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
}

impl Model {
    pub fn new(id: u32, origin: Vector3<f32>, rotation: f32, hull: Vec<Vector2<f32>>) -> Self {
        Self {
            id,
            origin,
            base_rotation: rotation,

            bounds: Bounds2D::new_containing(&hull),
            hull,

            offset: Vector2::zeros(),
            rotation: 0.0,
        }
    }

    // rotate hull around origin
    pub fn rotate(&mut self, angle: f32) {
        let origin = self.origin.xy();
        let rotation = Rotation2::new(angle - self.rotation);
        for point in self.hull.iter_mut() {
            let model_space = *point - origin;
            *point = (rotation * model_space) + origin;
        }

        self.rotation = angle;
        self.bounds = Bounds2D::new_containing(&self.hull);
    }
}

impl Objective {
    pub const ALL: [Self; 2] = [Self::Area, Self::Perimeter];

    pub fn name(&self) -> &str {
        match self {
            Objective::Area => "Area",
            Objective::Perimeter => "Perimeter",
        }
    }

    pub fn eval(&self, platform: Vector2<f32>, bounds: Bounds2D) -> f32 {
        let size = bounds.size();
        let mut score = match self {
            Objective::Area => size.x * size.y,
            Objective::Perimeter => size.x + size.y,
        };

        let (x, y) = (size.x > platform.x, size.y > platform.y);
        if x && y {
            score += 10_000.0 * (size.x * size.y - platform.x * platform.y);
        } else if x {
            score += 10_000.0 * (size.x * size.y - platform.x * size.y);
        } else if y {
            score += 10_000.0 * (size.x * size.y - size.x * platform.y);
        }

        score
    }
}

fn intersect_lines(start: Vector2<f32>, lines: &[Vector2<f32>]) -> usize {
    let mut count = 0;
    for (a, b) in lines.iter().chain(iter::once(&lines[0])).tuple_windows() {
        if (a.y > start.y) ^ (b.y > start.y) {
            let intersect_x = (b.x - a.x) * (start.y - a.y) / (b.y - a.y) + a.x;
            count += (start.x < intersect_x) as usize;
        }
    }

    count
}

fn area(polygon: &[Vector2<f32>]) -> f32 {
    let mut j = polygon.len() - 1;
    let mut area = 0.0;
    for i in 0..polygon.len() {
        area += (polygon[j].x + polygon[i].x) * (polygon[j].y - polygon[i].y);
        j = i;
    }

    area.abs() / 2.0
}
