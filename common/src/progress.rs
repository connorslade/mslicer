use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct Progress(Arc<ProgressInner>);

struct ProgressInner {
    complete: AtomicU64,
    total: AtomicU64,
}

impl Progress {
    pub fn new() -> Self {
        Self(Arc::new(ProgressInner {
            complete: AtomicU64::new(0),
            total: AtomicU64::new(0),
        }))
    }

    pub fn progress(&self) -> f32 {
        let total = self.0.total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }

        self.0.complete.load(Ordering::Relaxed) as f32 / total as f32
    }

    pub fn complete(&self) -> bool {
        let total = self.0.total.load(Ordering::Relaxed);
        if total == 0 {
            return false;
        }

        self.0.complete.load(Ordering::Relaxed) >= total
    }

    pub fn set_total(&self, total: u64) {
        self.0.total.store(total, Ordering::Relaxed);
    }

    pub fn set_complete(&self, complete: u64) {
        self.0.complete.store(complete, Ordering::Relaxed);
    }

    pub fn set_finished(&self) {
        let total = self.0.total.load(Ordering::Relaxed);
        self.0.complete.store(total, Ordering::Relaxed);
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}
