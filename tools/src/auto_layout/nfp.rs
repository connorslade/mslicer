use std::{cmp::Reverse, f32::consts::TAU};

use common::{geometry::convex_hull, progress::Progress};
use itertools::Itertools;
use nalgebra::{Vector2, Vector3};
use ordered_float::OrderedFloat;

use crate::auto_layout::{Model, area, bounds::Bounds2D, intersect_lines};

pub struct AutoLayoutNFP {
    padding: f32,
    segment_steps: f32,

    platform_size: Vector2<f32>,
    models: Vec<Model>,
}

impl AutoLayoutNFP {
    pub fn new(platform_size: Vector2<f32>, mut models: Vec<Model>) -> Self {
        models.sort_by_cached_key(|x| Reverse(OrderedFloat(area(&x.hull))));
        Self {
            padding: 0.0,
            segment_steps: 0.0,
            platform_size,
            models,
        }
    }

    pub fn new_unsorted(platform_size: Vector2<f32>, models: Vec<Model>) -> Self {
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

    // Strict mode will abort if not all models can be fit, which is needed when
    // using simulated annealing on top of this.
    pub fn layout(
        mut self,
        strict: bool,
        progress: Progress,
    ) -> Option<(f32, Vec<(u32, Vector3<f32>)>)> {
        progress.set_total(self.models.len() as _);

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
                            let total_bounds = bounds.include_bound(this.bounds.offset(p));

                            let size = total_bounds.size();
                            let objective = size.x * size.y;
                            if (size.x <= self.platform_size.x && size.y <= self.platform_size.y)
                                && objective < best.1
                            {
                                best = (p, objective);
                            }
                        }
                    }
                }
            }

            if best.1 != f32::MAX {
                bounds = bounds.include_bound(this.bounds.offset(best.0));
                let model = &mut self.models[i];
                model.hull.iter_mut().for_each(|x| *x += best.0);
                model.offset = best.0;
            } else if strict {
                return None;
            }
        }

        let bounds = (self.models.iter())
            .map(|x| x.bounds.offset(x.offset))
            .sum::<Bounds2D>();
        let size = bounds.size();
        if strict && (size.x > self.platform_size.x || size.y > self.platform_size.y) {
            return None;
        }

        let global_offset = -(bounds.min + bounds.size() / 2.0);

        progress.set_finished();
        let models = (self.models.into_iter())
            .map(|x| (x.id, x.origin + (x.offset + global_offset).to_homogeneous()))
            .collect();

        Some((size.x * size.y, models))
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
    convex_hull(&points)
}

fn offset(points: &[Vector2<f32>], d: f32, n: usize) -> Vec<Vector2<f32>> {
    let points = (points.iter())
        .cartesian_product(disk(n, d).iter())
        .map(|(i, j)| *i + *j)
        .collect::<Vec<_>>();
    convex_hull(&points)
}

fn disk(n: usize, r: f32) -> Vec<Vector2<f32>> {
    (0..n)
        .map(|i| {
            let (y, x) = (i as f32 / n as f32 * TAU).sin_cos();
            Vector2::new(x, y) * r
        })
        .collect()
}
