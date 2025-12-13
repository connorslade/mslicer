use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use nalgebra::Vector3;

pub mod stl;

#[derive(Default)]
pub struct Mesh {
    pub verts: Vec<Vector3<f32>>,
    pub faces: Vec<[u32; 3]>,
}

#[derive(Clone)]
pub struct Progress(Arc<ProgressInner>);

struct ProgressInner {
    complete: AtomicU32,
    total: AtomicU32,
}

impl Progress {
    pub fn new() -> Self {
        Self(Arc::new(ProgressInner {
            complete: AtomicU32::new(0),
            total: AtomicU32::new(0),
        }))
    }

    pub fn progress(&self) -> f32 {
        let total = self.0.total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }

        self.0.complete.load(Ordering::Relaxed) as f32 / total as f32
    }

    fn set_total(&self, total: u32) {
        self.0.total.store(total, Ordering::Relaxed);
    }

    fn set_complete(&self, complete: u32) {
        self.0.complete.store(complete, Ordering::Relaxed);
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}
