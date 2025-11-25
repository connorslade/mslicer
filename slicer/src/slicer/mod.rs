use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Condvar, Mutex,
    },
};

use common::{config::SliceConfig, format::Format};

use crate::{format::FormatSliceResult, mesh::Mesh};

mod slice_raster;
mod slice_vector;

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
            Format::Ctb => FormatSliceResult::Ctb(self.slice::<ctb_format::LayerEncoder>()),
            Format::Svg => FormatSliceResult::Svg(self.slice_vector()),
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
