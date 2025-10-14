use std::{collections::HashSet, sync::atomic::Ordering};

use common::misc::{VectorLayer, VectorSliceResult};
use nalgebra::Vector2;
use ordered_float::OrderedFloat;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    intersection::Segments1D,
    slicer::{Slicer, SEGMENT_LAYERS},
};

impl Slicer {
    pub fn slice_vector(&self) -> VectorSliceResult<'_> {
        let segments = self
            .models
            .iter()
            .map(|x| Segments1D::from_mesh(x, SEGMENT_LAYERS))
            .collect::<Vec<_>>();

        let layers = (0..self.progress.total)
            .into_par_iter()
            .inspect(|_| {
                self.progress.completed.fetch_add(1, Ordering::Relaxed);
                self.progress.notify.notify_all();
            })
            .map(|layer| {
                let height = layer as f32 * self.slice_config.slice_height;

                let segments = self
                    .models
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, mesh)| segments[idx].intersect_plane(mesh, height))
                    .flat_map(|x| x.0)
                    .map(|x| x.xy())
                    .collect::<Vec<_>>();

                VectorLayer {
                    polygons: join_segments(&segments),
                }
            })
            .collect::<Vec<_>>();

        self.progress.notify.notify_all();

        VectorSliceResult {
            layers,
            slice_config: &self.slice_config,
        }
    }
}

// this can be made more efficient with some kinda spacial partitioning system
// (it's currently like n²), but it's like whatever. its fast enough for what
// i'm doing.
fn join_segments(segments_raw: &[Vector2<f32>]) -> Vec<Vec<Vector2<f32>>> {
    const DISTANCE_CUTOFF: f32 = 0.5;

    let mut segments = HashSet::new();
    for segment in segments_raw.chunks_exact(2) {
        segments.insert((segment[0].map(OrderedFloat), segment[1].map(OrderedFloat)));
    }

    let mut polygons = Vec::new();
    while let Some(&start) = segments.iter().next() {
        let idx = polygons.len();
        polygons.push(Vec::new());

        let mut last = start.1;
        while let Some((x @ (a, b), [a_dist, b_dist])) = segments
            .iter()
            .map(|x @ (a, b)| (x, [a, b].map(|x| (last - x).map(|x| *x).magnitude())))
            .min_by_key(|(_, [a, b])| OrderedFloat(a.min(*b)))
        {
            let next = if a_dist < b_dist { b } else { a };
            if *next == start.1 || (a_dist > DISTANCE_CUTOFF && b_dist > DISTANCE_CUTOFF) {
                segments.remove(&{ *x });
                break;
            }

            polygons[idx].push(next.map(|x| *x));
            last = *next;
            segments.remove(&{ *x });
        }
    }

    polygons
}
