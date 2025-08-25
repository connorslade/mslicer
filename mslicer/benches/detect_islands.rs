use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

use mslicer::post_processing::{IslandDetectionPass, Pass, PassProgress};
use parking_lot::Mutex;
use rayon::ThreadPoolBuilder;

mod common;

pub fn benchmark_island_detection(_: &mut Criterion) {
    ThreadPoolBuilder::new()
        .num_threads(std::thread::available_parallelism().unwrap().get() - 1)
        .build_global()
        .unwrap();
    let (cfg, slice_res) = common::prepare();
    let prg = Arc::new(Mutex::new(PassProgress::default()));
    let mut crit = Criterion::default().sample_size(20);
    crit.bench_function("detect_islands", |b| {
        b.iter(|| {
            let mut pass = IslandDetectionPass::default();
            let sr = Arc::new(Mutex::new(Some(slice_res.clone())));
            pass.run(&cfg, sr, prg.clone()).join().unwrap();
        });
    });
}

criterion_group!(island_detection, benchmark_island_detection);
criterion_main!(island_detection);
