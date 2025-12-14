use std::{fs::File, io::BufReader, path::Path};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use slicer::{intersection::Segments1D, mesh::load_mesh};

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mesh Intersections");

    let parent = Path::new("..");
    for mesh_name in ["teapot.stl", "dragon.stl", "david.stl"] {
        let file = BufReader::new(File::open(parent.join(mesh_name)).unwrap());
        let mesh = load_mesh(file, "stl").unwrap();
        let segments = Segments1D::from_mesh(&mesh, 100);

        group.bench_with_input(BenchmarkId::new("Linear", mesh_name), &mesh, |b, i| {
            b.iter(|| i.intersect_plane(0.0))
        });

        group.bench_with_input(
            BenchmarkId::new("Segments", mesh_name),
            &(segments, mesh),
            |b, (segments, mesh)| b.iter(|| segments.intersect_plane(mesh, 0.0)),
        );
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
