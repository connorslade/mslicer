use std::iter;

use itertools::Itertools;
use nalgebra::{Vector2, Vector3};

use crate::auto_layout::bounds::Bounds2D;

mod bounds;
mod nfp;
pub use self::nfp::AutoLayoutNFP;

pub struct Model {
    id: u32,
    origin: Vector3<f32>,
    bounds: Bounds2D,
    hull: Vec<Vector2<f32>>,
    offset: Vector2<f32>,
}

impl Model {
    pub fn new(id: u32, origin: Vector3<f32>, hull: Vec<Vector2<f32>>) -> Self {
        Self {
            id,
            origin,
            bounds: Bounds2D::new_containing(&hull),
            hull,
            offset: Vector2::zeros(),
        }
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
