use std::{fs::File, io::BufReader, path::Path};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use slicer::{mesh::load_mesh, segments::Segments1D};

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mesh Intersections");

    let parent = Path::new("..");
    for mesh_name in ["teapot.stl", "dragon.stl", "david.stl"] {
        let mut file = BufReader::new(File::open(parent.join(mesh_name)).unwrap());
        let mut mesh = load_mesh(&mut file, "stl").unwrap();
        mesh.recompute_normals();
        let segments = Segments1D::from_mesh(&mesh, 100);

        group.bench_with_input(BenchmarkId::new("Linier", mesh_name), &mesh, |b, i| {
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
