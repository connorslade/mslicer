use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Condvar, Mutex,
    },
};

use common::{
    config::SliceConfig,
    misc::{EncodableLayer, SliceResult},
};
use itertools::Itertools;
use nalgebra::Vector3;
use ordered_float::OrderedFloat;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{bvh::Bvh, mesh::Mesh, segments::Segments, Pos};

pub struct Slicer {
    slice_config: SliceConfig,
    models: Vec<Mesh>,
    progress: Progress,
}

#[derive(Clone)]
pub struct Progress {
    inner: Arc<ProgressInner>,
}

pub struct ProgressInner {
    completed: AtomicU32,
    total: u32,

    notify: Condvar,
    last_completed: Mutex<u32>,
}

impl Slicer {
    pub fn new(slice_config: SliceConfig, models: Vec<Mesh>) -> Self {
        let max = models.iter().fold(Pos::zeros(), |max, model| {
            let f = model.vertices.iter().fold(Pos::zeros(), |max, &f| {
                let f = model.transform(&f);
                Pos::new(max.x.max(f.x), max.y.max(f.y), max.z.max(f.z))
            });
            Pos::new(max.x.max(f.x), max.y.max(f.y), max.z.max(f.z))
        });
        let layers = (max.z / slice_config.slice_height).ceil() as u32;

        Self {
            slice_config,
            models,
            progress: Progress {
                inner: Arc::new(ProgressInner {
                    completed: AtomicU32::new(0),
                    total: layers,

                    notify: Condvar::new(),
                    last_completed: Mutex::new(0),
                }),
            },
        }
    }

    pub fn progress(&self) -> Progress {
        self.progress.clone()
    }

    pub fn slice<Layer: EncodableLayer>(&self) -> SliceResult<Layer::Output> {
        let segments = self
            .models
            .iter()
            .map(|x| Segments::from_mesh(x, 100))
            .collect::<Vec<_>>();

        let layers = (0..self.progress.total)
            .into_par_iter()
            .inspect(|_| {
                self.progress.completed.fetch_add(1, Ordering::Relaxed);
                self.progress.notify.notify_all();
            })
            .map(|layer| {
                let height = layer as f32 * self.slice_config.slice_height;

                let intersections = self
                    .models
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, mesh)| segments[idx].intersect_plane(mesh, height));
                let segments = intersections
                    .chunks(2)
                    .into_iter()
                    .map(|mut x| (x.next().unwrap(), x.next().unwrap()))
                    .collect::<Vec<_>>();

                let mut encoder = Layer::new();
                let mut last = 0;

                for y in 0..self.slice_config.platform_resolution.y {
                    let yf = y as f32;
                    let mut intersections = segments
                        .iter()
                        .filter(|&(a, b)| ((a.y > yf) ^ (b.y > yf)))
                        .map(|(a, b)| {
                            let t = (yf - a.y) / (b.y - a.y);
                            a.x + t * (b.x - a.x)
                        })
                        .collect::<Vec<_>>();

                    intersections.sort_by_key(|&x| OrderedFloat(x));

                    for span in intersections.chunks_exact(2) {
                        let y_offset = (self.slice_config.platform_resolution.x * y) as u64;

                        let a = span[0].round() as u64;
                        let b = span[1].round() as u64;

                        let start = a + y_offset;
                        let end = b + y_offset;
                        let length = b - a;

                        if start > last {
                            encoder.add_run(start - last, 0);
                        }

                        encoder.add_run(length, 255);
                        last = end;
                    }
                }

                encoder.finish(layer as usize, &self.slice_config)
            })
            .collect::<Vec<_>>();

        self.progress.notify.notify_all();

        SliceResult {
            layers,
            slice_config: &self.slice_config,
        }
    }
}

impl Deref for Progress {
    type Target = ProgressInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Progress {
    pub fn wait(&self) -> u32 {
        let mut last_completed = self
            .notify
            .wait(self.last_completed.lock().unwrap())
            .unwrap();

        let current = self.completed.load(Ordering::Relaxed);
        if *last_completed < current {
            *last_completed = current;
        }

        current
    }

    pub fn completed(&self) -> u32 {
        self.completed.load(Ordering::Relaxed)
    }

    pub fn total(&self) -> u32 {
        self.total
    }
}
