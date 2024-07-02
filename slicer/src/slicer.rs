use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Condvar, Mutex,
    },
};

use common::{config::SliceConfig, image::Image, misc::SliceResult};
use ordered_float::OrderedFloat;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{mesh::Mesh, Pos};

pub struct Slicer {
    slice_config: SliceConfig,
    model: Mesh,
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
    pub fn new(slice_config: SliceConfig, model: Mesh) -> Self {
        let max = model.vertices.iter().fold(Pos::zeros(), |max, &f| {
            let f = model.transform(&f);
            Pos::new(max.x.max(f.x), max.y.max(f.y), max.z.max(f.z))
        });
        let layers = (max.z / slice_config.slice_height).ceil() as u32;

        Self {
            slice_config,
            model,
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

    pub fn slice(&self) -> SliceResult {
        let (slice_config, model) = (&self.slice_config, &self.model);
        let layers = (0..self.progress.total)
            .into_par_iter()
            .inspect(|_| {
                self.progress.completed.fetch_add(1, Ordering::Relaxed);
                self.progress.notify.notify_all();
            })
            .map(|layer| {
                let height = layer as f32 * slice_config.slice_height;
                let intersections = model.intersect_plane(height);

                let segments = intersections
                    .chunks(2)
                    .map(|x| (x[0], x[1]))
                    .collect::<Vec<_>>();

                let mut out = Vec::new();
                for y in 0..slice_config.platform_resolution.y {
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
                        let y_offset = (slice_config.platform_resolution.x * y) as u64;
                        out.push((
                            y_offset + span[0].round() as u64,
                            y_offset + span[1].round() as u64,
                        ));
                    }
                }

                let mut image = Image::blank(
                    self.slice_config.platform_resolution.x as usize,
                    self.slice_config.platform_resolution.y as usize,
                );

                let mut last = 0;
                for (start, end) in out {
                    if start > last {
                        image.add_run((start - last) as usize, 0);
                    }

                    assert!(end >= start, "End precedes start in layer {layer}");
                    image.add_run((end - start) as usize, 255);
                    last = end;
                }

                image
            })
            .collect::<Vec<_>>();

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
