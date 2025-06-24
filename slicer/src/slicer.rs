use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Condvar, Mutex,
    },
};

use common::{
    config::SliceConfig,
    format::Format,
    misc::{EncodableLayer, SliceResult, VectorLayer, VectorSliceResult},
};
use ordered_float::OrderedFloat;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{format::FormatSliceResult, mesh::Mesh, segments::Segments1D};

const SEGMENT_LAYERS: usize = 100;

/// Used to slice a mesh.
pub struct Slicer {
    slice_config: SliceConfig,
    models: Vec<Mesh>,
    progress: Progress,
}

/// Allows checking the progress of a slicing operation.
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
    /// Creates a new slicer given a slice config and list of models.
    pub fn new(slice_config: SliceConfig, models: Vec<Mesh>) -> Self {
        let max_z = models.iter().fold(0_f32, |max, model| {
            let verts = model.vertices().iter();
            let z = verts.fold(0_f32, |max, &f| max.max(model.transform(&f).z));
            max.max(z)
        });

        let layers = (max_z / slice_config.slice_height).ceil() as u32;
        let max_layers = (slice_config.platform_size.z / slice_config.slice_height).ceil() as u32;

        Self {
            slice_config,
            models,
            progress: Progress {
                inner: Arc::new(ProgressInner {
                    completed: AtomicU32::new(0),
                    total: layers.min(max_layers),

                    notify: Condvar::new(),
                    last_completed: Mutex::new(0),
                }),
            },
        }
    }

    /// Gets an instance of the slicing [`Progress`] struct.
    pub fn progress(&self) -> Progress {
        self.progress.clone()
    }

    pub fn slice_format(&self) -> FormatSliceResult<'_> {
        match self.slice_config.format {
            Format::Goo => FormatSliceResult::Goo(self.slice::<goo_format::LayerEncoder>()),
            Format::Svg => FormatSliceResult::Svg(self.slice_vector()),
        }
    }

    /// Actually runs the slicing operation, it is multithreaded.
    pub fn slice<Layer: EncodableLayer>(&self) -> SliceResult<'_, Layer::Output> {
        let platform_resolution = self.slice_config.platform_resolution;
        let pixels = (platform_resolution.x * platform_resolution.y) as u64;

        // A segment contains a reference to all of the triangles it contains. By
        // splitting the mesh into segments, not all triangles need to be tested
        // to find all intersections. This massively speeds up the slicing
        // operation and actually makes it faster than most other slicers. :p
        let segments = self
            .models
            .iter()
            .map(|x| Segments1D::from_mesh(x, SEGMENT_LAYERS))
            .collect::<Vec<_>>();

        let layers = (0..self.progress.total)
            .into_par_iter()
            .inspect(|_| {
                // Updates the slice progress
                self.progress.completed.fetch_add(1, Ordering::Relaxed);
                self.progress.notify.notify_all();
            })
            .map(|layer| {
                let height = layer as f32 * self.slice_config.slice_height;

                // Gets all the intersections between the slice plane and the
                // model. Because all the faces are triangles, every triangle
                // intersection will return two points. These can then be
                // interpreted as line segments making up a polygon.
                let segments = self
                    .models
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, mesh)| segments[idx].intersect_plane(mesh, height))
                    .collect::<Vec<_>>();

                // Creates a new encoded for this layer. Because printers can
                // have very high resolution displays, the uncompressed data for
                // a sliced model can easily be over 30 Gigabytes. Most formats
                // use some sort of compression scheme to resolve this issue, so
                // to use a little memory as needed, the layers are compressed
                // as they are made.
                let mut encoder = Layer::new();
                let mut last = 0;

                // For each row of pixels, we find all line segments that go
                // across and mark that as an intersection to then be run-length
                // encoded. There is probably a better polygon filling algo, but
                // this one works surprisingly fast.
                for y in 0..platform_resolution.y {
                    let yf = y as f32;
                    let mut intersections = segments
                        .iter()
                        .map(|x| (x.0[0], x.0[1], x.1))
                        // Filtering to only consider segments with one point
                        // above the current row and one point below.
                        .filter(|&(a, b, _)| ((a.y > yf) ^ (b.y > yf)))
                        .map(|(a, b, facing)| {
                            // Get the x position of the line segment at this y
                            let t = (yf - a.y) / (b.y - a.y);
                            (a.x + t * (b.x - a.x), facing)
                        })
                        .collect::<Vec<_>>();

                    // Sort all these intersections for run-length encoding
                    intersections.sort_by_key(|&(x, _)| OrderedFloat(x));

                    // In order to avoid creating a cavity in the model when
                    // there is an intersection either by the same mesh or
                    // another mesh, these intersections are removed. This is
                    // done by looking at the direction each line segment is
                    // facing. For example, <- <- -> -> would be reduced to <- ->.
                    let mut filtered = Vec::with_capacity(intersections.len());
                    let mut depth = 0;

                    for (pos, dir) in intersections {
                        let prev_depth = depth;
                        depth += (dir as i32) * 2 - 1;

                        if (depth == 0) ^ (prev_depth == 0) {
                            filtered.push(pos.clamp(0.0, platform_resolution.x as f32));
                        }
                    }

                    // Convert the intersections into runs of white pixels to be
                    // encoded into the layer
                    for span in filtered.chunks_exact(2) {
                        let y_offset = (platform_resolution.x * y) as u64;

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

                // Turns out that on my printer, the buffer that each layer is
                // decoded into is just uninitialized memory. So if the last run
                // doesn't fill the buffer, the printer will just print whatever
                // was in the buffer before which just makes a huge mess.
                if last < pixels {
                    encoder.add_run(pixels - last, 0);
                }

                // Finished encoding the layer
                encoder.finish(layer as usize, &self.slice_config)
            })
            .collect::<Vec<_>>();

        self.progress.notify.notify_all();

        SliceResult {
            layers,
            slice_config: &self.slice_config,
        }
    }

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
                    .collect::<Vec<_>>();

                VectorLayer {
                    points: segments
                        .into_iter()
                        // todo: skipping 2nd point
                        .map(|(points, _side)| points[0].xy())
                        .collect(),
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

impl Deref for Progress {
    type Target = ProgressInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Progress {
    /// Waits until the next layer is complete, returning the current count of
    /// sliced layers.
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

    /// Returns the count of sliced layers.
    pub fn completed(&self) -> u32 {
        self.completed.load(Ordering::Relaxed)
    }

    /// Returns the count of layers in the current slicing operation.
    pub fn total(&self) -> u32 {
        self.total
    }
}
