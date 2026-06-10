use std::{cmp::Reverse, f32::consts::TAU, fs, iter};

use common::{geometry::convex_hull, progress::Progress};
use itertools::Itertools;
use nalgebra::Vector2;
use ordered_float::OrderedFloat;

use crate::{auto_layout::bounds::Bounds2D, printed_circuit_board::polygons::Polygons};

mod bounds;

pub struct AutoLayout {
    padding: f32,
    segment_steps: f32,

    platform_size: Vector2<f32>,
    models: Vec<Model>,
}

pub struct Model {
    id: u32,
    bounds: Bounds2D,
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

        let mut bounds = Bounds2D::EMPTY;
        for i in 1..self.models.len() {
            progress.add_complete(1);
            let this = &self.models[i];
            let nfps = (self.models.iter().take(i))
                .map(|x| non_fitting_polygon(x.hull.iter(), this.hull.iter()))
                .map(|x| offset(&x, self.padding, 6))
                .collect::<Vec<_>>();

            // pick one of the points that is outside all nfps.
            let mut best = (Vector2::repeat(f32::MAX), f32::MAX);
            for j in 0..i {
                let nfp = &nfps[j];
                for (pa, pb) in nfp.iter().tuple_windows() {
                    let vector = pb - pa;
                    let norm = vector.normalize();
                    let n = (vector.magnitude() * self.segment_steps).ceil() as usize;
                    for k in 0..=n {
                        let p = pa + norm * (k as f32 / n as f32);
                        let valid = (nfps.iter().take(i).enumerate())
                            .all(|(i, x)| i == j || intersect_lines(p, x) & 1 == 0);
                        if valid {
                            let new_bounds = this.bounds + p;
                            let total_bounds = bounds + new_bounds;

                            let size = total_bounds.size();
                            let objective = size.x.max(size.y);
                            if size <= self.platform_size && objective < best.1 {
                                best = (p, objective);
                                bounds = total_bounds;
                            }
                        }
                    }
                }
            }

            if best.1 != f32::MAX {
                let model = &mut self.models[i];
                model.hull.iter_mut().for_each(|x| *x += best.0);
                model.offset = best.0;
            } else {
                println!("No placements found");
            }
        }

        for m in self.models.iter() {
            debug.trace(m.hull.iter().map(|x| x.cast::<f64>()).collect(), Some(1.0));
        }

        debug.circle(Vector2::zeros(), 0.5);
        fs::write("debug.svg", debug.svg()).unwrap();

        let bounds = (self.models.iter())
            .map(|x| x.bounds + x.offset)
            .sum::<Bounds2D>();
        let global_offset = -(bounds.min + bounds.size() / 2.0);

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
            bounds: Bounds2D::new_containing(&hull),
            hull,
            offset: Vector2::zeros(),
        }
    }
}

fn non_fitting_polygon<'a>(
    a: impl Iterator<Item = &'a Vector2<f32>>,
    b: impl Iterator<Item = &'a Vector2<f32>> + Clone,
) -> Vec<Vector2<f32>> {
    let points = a
        .cartesian_product(b)
        .map(|(i, j)| *i - *j)
        .collect::<Vec<_>>();
    convex_hull(&points).into_iter().copied().collect()
}

fn offset(points: &[Vector2<f32>], d: f32, n: usize) -> Vec<Vector2<f32>> {
    let points = (points.iter())
        .cartesian_product(disk(n, d).iter())
        .map(|(i, j)| *i + *j)
        .collect::<Vec<_>>();
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

fn disk(n: usize, r: f32) -> Vec<Vector2<f32>> {
    (0..n)
        .map(|i| {
            let (y, x) = (i as f32 / n as f32 * TAU).sin_cos();
            Vector2::new(x, y) * r
        })
        .collect()
}
