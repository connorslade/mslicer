use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
};

use clone_macro::clone;
use common::serde::Deserializer;
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
    complete: AtomicU64,
    total: AtomicU64,
}

pub fn load_mesh<T: Deserializer + Send + 'static>(
    mut des: T,
    format: &str,
) -> (Progress, Receiver<Mesh>) {
    let progress = Progress::new();
    let (tx, rx) = mpsc::sync_channel(1);

    let format = format.to_ascii_lowercase();
    match format.as_str() {
        "stl" => {
            thread::spawn(clone!([progress], move || {
                let mesh = stl::parse(&mut des, progress.clone()).unwrap();
                tx.send(mesh).unwrap();
                progress.set_finished();
            }));
        }
        _ => panic!("Unsupported format: {}", format),
    }

    (progress, rx)
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

    fn set_total(&self, total: u64) {
        self.0.total.store(total, Ordering::Relaxed);
    }

    fn set_complete(&self, complete: u64) {
        self.0.complete.store(complete, Ordering::Relaxed);
    }

    fn set_finished(&self) {
        let total = self.0.total.load(Ordering::Relaxed);
        self.0.complete.store(total, Ordering::Relaxed);
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}
