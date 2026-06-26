use std::iter;

use common::progress::Progress;
use itertools::Itertools;
use nalgebra::Vector2;
use tracing::warn;

use crate::auto_layout::{
    Model, Objective, Placement,
    bounds::Bounds2D,
    cache::{CacheEntry, LayoutCache},
};

pub struct AutoLayoutNfp<'a> {
    objective: Objective,
    segment_steps: f32,
    bounds_penalty: f32,

    platform_size: Vector2<f32>,
    cache: &'a mut LayoutCache,
    models: Vec<Model>,
}

impl<'a> AutoLayoutNfp<'a> {
    pub fn new(
        platform_size: Vector2<f32>,
        models: Vec<Model>,
        cache: &'a mut LayoutCache,
    ) -> Self {
        // todo: sort models where appropriate
        // models.sort_by_cached_key(|x| Reverse(OrderedFloat(area(&x.hull))));
        Self {
            objective: Objective::Area,
            segment_steps: 10.0,
            bounds_penalty: 10_000.0,
            platform_size,
            models,
            cache,
        }
    }

    pub fn segment_steps(self, segment_steps: f32) -> Self {
        Self {
            segment_steps,
            ..self
        }
    }

    pub fn bounds_penalty(self, bounds_penalty: f32) -> Self {
        Self {
            bounds_penalty,
            ..self
        }
    }

    pub fn objective(self, objective: Objective) -> Self {
        Self { objective, ..self }
    }

    fn eval(&self, bounds: Bounds2D) -> f32 {
        self.objective
            .eval(self.platform_size, self.bounds_penalty, bounds)
    }

    pub fn layout(mut self, progress: Progress) -> Option<(f32, Vec<Placement>)> {
        progress.set_total(self.models.len() as _);

        let first = &self.models[0];
        let first_hull = self.cache.hull(&first.entry());
        let mut bounds = first_hull.bounds.offset(first.position);

        for i in 1..self.models.len() {
            progress.add_complete(1);
            let this_entry = self.models[i].entry();
            let this_hull = self.cache.hull(&this_entry);

            // pick one of the points that is outside all nfps.
            let mut best = (Vector2::repeat(f32::MAX), f32::MAX, Bounds2D::EMPTY);
            for j in 0..i {
                let position = self.models[j].position;
                let orbiting_entry = self.models[j].entry();
                let nfp = self.cache.nfp(orbiting_entry, this_entry);

                for (pa, pb) in nfp.iter().chain(iter::once(&nfp[0])).tuple_windows() {
                    let vector = pb - pa;
                    let n = (vector.magnitude() * self.segment_steps).ceil() as usize;
                    for k in 0..=n {
                        let p = pa + vector * (k as f32 / n as f32) + position;
                        let valid = (self.models.iter().take(i).enumerate()).all(|(i, x)| {
                            i == j || intersect_nfp(&mut self.cache, p, x, this_entry) & 1 == 0
                        });
                        if valid {
                            let total_bounds = bounds.include_bound(this_hull.bounds.offset(p));

                            let objective = self.eval(total_bounds);
                            (objective < best.1).then(|| best = (p, objective, total_bounds));
                        }
                    }
                }
            }

            if best.1 != f32::MAX {
                bounds = best.2;
                self.models[i].position = best.0;
            } else {
                warn!("No placement found");
            }
        }

        let bounds = (self.models.iter())
            .map(|x| self.cache.hull(&x.entry()).bounds.offset(x.position))
            .sum::<Bounds2D>();
        let global_offset = -(bounds.min + bounds.size() / 2.0);

        progress.set_finished();
        let models = (self.models.iter())
            .map(|x| Placement {
                model: x.model,
                position: (x.position + global_offset).to_homogeneous(),
                rotation: x.rotation,
            })
            .collect();

        Some((self.eval(bounds), models))
    }
}

// todo: intersect bounding boxes first?
fn intersect_nfp(
    cache: &mut LayoutCache,
    start: Vector2<f32>,
    orbiting: &Model,
    entry: CacheEntry,
) -> usize {
    let nfp = cache.nfp(orbiting.entry(), entry);

    let mut count = 0;
    for (a, b) in nfp.iter().chain(iter::once(&nfp[0])).tuple_windows() {
        let (a, b) = (a + orbiting.position, b + orbiting.position);
        if (a.y > start.y) ^ (b.y > start.y) {
            let intersect_x = (b.x - a.x) * (start.y - a.y) / (b.y - a.y) + a.x;
            count += (start.x < intersect_x) as usize;
        }
    }

    count
}
