use std::{
    array,
    ops::Index,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone)]
pub struct Progress(Arc<ProgressInner>);

struct ProgressInner {
    complete: AtomicU64,
    total: AtomicU64,
}

#[derive(Clone)]
pub struct CombinedProgress<const N: usize> {
    inner: [Progress; N],
}

impl Progress {
    pub fn new() -> Self {
        Self(Arc::new(ProgressInner {
            complete: AtomicU64::new(0),
            total: AtomicU64::new(0),
        }))
    }

    pub fn get_complete(&self) -> u64 {
        self.0.complete.load(Ordering::Relaxed)
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

    pub fn add_complete(&self, delta: u64) {
        self.0.complete.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn set_finished(&self) {
        let total = self.0.total.load(Ordering::Relaxed);
        self.0.complete.store(total, Ordering::Relaxed);
    }
}

impl<const N: usize> CombinedProgress<N> {
    pub fn new() -> Self {
        Self {
            inner: array::from_fn(|_| Progress::new()),
        }
    }

    pub fn count(&self) -> usize {
        N
    }

    pub fn progress(&self) -> f32 {
        self.inner.iter().map(|x| x.progress()).sum::<f32>() / N as f32
    }

    pub fn complete(&self) -> bool {
        self.inner.iter().all(|x| x.complete())
    }
}

impl<const N: usize> Index<usize> for CombinedProgress<N> {
    type Output = Progress;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Default for CombinedProgress<N> {
    fn default() -> Self {
        Self::new()
    }
}
