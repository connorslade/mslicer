use std::sync::Arc;

use ::common::annotations::{Annotations, ClusterView};
use criterion::{criterion_group, criterion_main, Criterion};
use mslicer::post_processing::{IslandDetectionPass, Pass, PassOutput, PassProgress};
use parking_lot::Mutex;
use rayon::ThreadPoolBuilder;

mod common;

#[allow(irrefutable_let_patterns)]
fn benchmark_annotation_clustering(_: &mut Criterion) {
    ThreadPoolBuilder::new()
        .num_threads(std::thread::available_parallelism().unwrap().get() - 1)
        .build_global()
        .unwrap();
    let (cfg, _) = common::prepare();
    let prg = Arc::new(Mutex::new(PassProgress::default()));
    let (_, slice_res) = common::prepare();
    let mut pass = IslandDetectionPass::default();
    let sr = Arc::new(Mutex::new(Some(slice_res.clone())));
    pass.run(&cfg, sr, prg.clone()).join().unwrap();
    let result = pass.result().lock().take().unwrap();
    let annos: Annotations = if let PassOutput::Analysis(report) = result {
        Some(
            report
                .annotations()
                .into_iter()
                .filter(|a| a.slice_idx().map(|idx| idx < 10).unwrap_or(false))
                .map(|a| a.clone())
                .collect::<Vec<_>>(),
        )
    } else {
        None
    }
    .unwrap()
    .into();

    let mut crit = Criterion::default().sample_size(10);
    crit.bench_function("cluster_annotations", move |b| {
        b.iter(|| {
            let cv = ClusterView::new(Arc::new(annos.clone()));
            println!("{} clusters", cv.clusters.len());
        });
    });
}

criterion_group!(clustering, benchmark_annotation_clustering);
criterion_main!(clustering);
