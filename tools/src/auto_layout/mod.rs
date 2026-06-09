use std::{cmp::Reverse, fs, iter};

use common::{geometry::convex_hull, progress::Progress};
use itertools::Itertools;
use nalgebra::Vector2;
use ordered_float::OrderedFloat;

use crate::printed_circuit_board::polygons::Polygons;

const MAX_BOUNDS: (Vector2<f32>, Vector2<f32>) = (
    Vector2::new(f32::MAX, f32::MAX),
    Vector2::new(f32::MIN, f32::MIN),
);

pub struct AutoLayout {
    padding: f32,
    segment_steps: f32,

    platform_size: Vector2<f32>,
    models: Vec<Model>,
}

pub struct Model {
    id: u32,
    bounds: (Vector2<f32>, Vector2<f32>),
    hull: Vec<Vector2<f32>>,
    offset: Vector2<f32>,
}

impl AutoLayout {
    pub fn new(platform_size: Vector2<f32>, mut models: Vec<Model>) -> Self {
        models.sort_by_cached_key(|x| Reverse(OrderedFloat(area(&x.hull))));
        Self {
            padding: 0.0,
            segment_steps: 0.0,
            platform_size,
            models,
        }
    }

    pub fn padding(self, padding: f32) -> Self {
        Self { padding, ..self }
    }

    pub fn segment_steps(self, segment_steps: f32) -> Self {
        Self {
            segment_steps,
            ..self
        }
    }

    pub fn layout(mut self, progress: Progress) -> Vec<(u32, Vector2<f32>)> {
        progress.set_total(self.models.len() as _);

        let mut debug = Polygons::new();
        let mut points = Vec::new();

        for i in 1..self.models.len() {
            progress.add_complete(1);
            let this = &self.models[i];
            let nfps = (self.models.iter().take(i))
                .map(|x| non_fitting_polygon(x.hull.iter(), this.hull.iter()))
                .collect::<Vec<_>>();

            // pick one of the points that is outside all nfps.
            let mut best = Vector2::repeat(f32::MAX);
            for j in 0..i {
                let nfp = &nfps[j];
                for (pa, pb) in nfp.iter().tuple_windows() {
                    let vector = pb - pa;
                    let norm = vector.normalize();
                    let n = (vector.magnitude() * self.segment_steps) as usize;
                    for k in 0..=n {
                        let p = pa + norm * (k as f32 / n as f32);
                        let valid = (nfps.iter().take(i).enumerate())
                            .all(|(i, x)| i == j || intersect_lines(p, x) & 1 == 0);
                        if valid {
                            let new_bounds = (this.bounds.0 + p, this.bounds.1 + p);
                            let total_bounds = (self.models.iter().take(i))
                                .map(|x| (x.bounds.0 + x.offset, x.bounds.1 + x.offset))
                                .chain(iter::once(new_bounds))
                                .fold(MAX_BOUNDS, |(min, max), v| {
                                    (min.zip_map(&v.0, f32::min), max.zip_map(&v.1, f32::max))
                                });

                            let size = total_bounds.1 - total_bounds.0;
                            (size <= self.platform_size
                                && p.magnitude_squared() < best.magnitude_squared())
                            .then(|| best = p);
                        }
                    }
                }
            }

            if best != Vector2::repeat(f32::MAX) {
                let model = &mut self.models[i];
                model.hull.iter_mut().for_each(|x| *x = *x + best);
                model.offset = best;
            } else {
                println!("No placements found");
            }
        }

        for m in self.models.iter() {
            for p in m.hull.iter() {
                points.push(*p);
            }
            debug.trace(points.iter().map(|x| x.cast::<f64>()).collect(), None);
            points.clear();
        }

        debug.circle(Vector2::zeros(), 0.5);
        fs::write("debug.svg", debug.svg()).unwrap();

        let bounds = (self.models.iter())
            .map(|x| (x.bounds.0 + x.offset, x.bounds.1 + x.offset))
            .fold(MAX_BOUNDS, |(min, max), v| {
                (min.zip_map(&v.0, f32::min), max.zip_map(&v.1, f32::max))
            });
        let size = bounds.1 - bounds.0;
        let global_offset = -(bounds.0 + size / 2.0);

        progress.set_finished();
        self.models
            .into_iter()
            .map(|x| (x.id, x.offset + global_offset))
            .collect()
    }
}

impl Model {
    pub fn new(id: u32, hull: Vec<Vector2<f32>>) -> Self {
        Self {
            id,
            bounds: bounds(&hull),
            hull,
            offset: Vector2::zeros(),
        }
    }
}

fn non_fitting_polygon<'a>(
    a: impl Iterator<Item = &'a Vector2<f32>>,
    b: impl Iterator<Item = &'a Vector2<f32>> + Clone,
) -> Vec<Vector2<f32>> {
    let mut points = Vec::new();
    for (i, j) in a.cartesian_product(b) {
        points.push(*i - *j);
    }

    convex_hull(&points).into_iter().copied().collect()
}

fn area(polygon: &[Vector2<f32>]) -> f32 {
    let mut j = polygon.len() - 1;
    let mut area = 0.0;
    for i in 0..polygon.len() {
        area += (polygon[j].x + polygon[i].x) + (polygon[j].y - polygon[i].y);
        j = i;
    }

    area.abs() / 2.0
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

fn bounds(vertices: &[Vector2<f32>]) -> (Vector2<f32>, Vector2<f32>) {
    vertices.iter().fold(MAX_BOUNDS, |(min, max), v| {
        (min.zip_map(&v, f32::min), max.zip_map(&v, f32::max))
    })
}
