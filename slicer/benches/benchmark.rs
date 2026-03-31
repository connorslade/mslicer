use common::{progress::Progress, serde::SliceDeserializer, slice::SliceConfig};
use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main, measurement::WallTime,
};
use slicer::{
    mesh::Mesh,
    slicer::{Slicer, SlicerModel},
};
use std::{fs, hint::black_box};

fn benchmark(c: &mut Criterion) {
    let g = &mut c.benchmark_group("slicer");
    slice_benchmark(g, "utah_teapot");
    slice_benchmark(g, "stanford_dragon");
    slice_benchmark(g, "stress_test");
}

criterion_group!(benches, benchmark);
criterion_main!(benches);

fn slicer(model: &[u8]) -> Slicer {
    let des = SliceDeserializer::new(model);
    let mesh = mesh_format::load_mesh(des, "stl", Progress::new()).unwrap();

    Slicer::new(
        SliceConfig::default(),
        vec![SlicerModel {
            mesh: Mesh::new(mesh.verts, mesh.faces),
            exposure: 255,
        }],
    )
}

fn slice_benchmark(g: &mut BenchmarkGroup<'_, WallTime>, name: &str) {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/benches/models/" + name + ".stl";
    let data = fs::read(path).unwrap();
    let slicer = slicer(&data);

    g.bench_function(name, |b| {
        b.iter(|| black_box(slicer.slice_raster::<goo_format::LayerEncoder>()));
    });
}
