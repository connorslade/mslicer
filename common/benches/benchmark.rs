use common::container::rle::png::{
    ColorType, PngEncoder, PngInfo,
    deflate::{Adler32, huffman, lz77_compress},
    intersperse_runs,
};
use common::{
    container::{BitVec, Run},
    serde::DynamicSerializer,
};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use nalgebra::Vector2;
use std::hint::black_box;

const PLATFORM: Vector2<u32> = Vector2::new(11520, 5120);

fn criterion_benchmark(c: &mut Criterion) {
    let layer = generate_layer();

    c.bench_function("Intersperse Runs", |b| {
        b.iter_batched(
            || layer.clone(),
            |mut runs| black_box(intersperse_runs(&mut runs, 0, 11520)),
            BatchSize::SmallInput,
        )
    });

    c.bench_function("Checksum", |b| {
        b.iter(|| {
            let mut check = Adler32::new();
            layer.iter().for_each(|run| check.update_run(run));
            black_box(check.finish())
        })
    });

    let mut rgb = layer.clone();
    intersperse_runs(&mut rgb, 0, 11520);

    c.bench_function("Tokens", |b| {
        b.iter_batched(
            || rgb.clone(),
            |rgb| black_box(lz77_compress(rgb.into_iter())),
            BatchSize::SmallInput,
        );
    });

    let tokens = lz77_compress(rgb.into_iter());
    c.bench_function("Huffman", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            let mut bits = BitVec::new(&mut out, 0);
            huffman(&mut bits, &tokens);
            black_box(out)
        });
    });

    c.bench_function("Layer Encode", |b| {
        b.iter_batched(
            || layer.clone(),
            |rgb| {
                let info = PngInfo {
                    width: PLATFORM.x / 3,
                    height: PLATFORM.y,
                    bit_depth: 8,
                    color_type: ColorType::Truecolor,
                };

                let mut ser = DynamicSerializer::new();
                let mut encoder = PngEncoder::new(&mut ser, &info, 3);
                encoder.write_pixel_dimensions(3, 1);
                encoder.write_image_data(rgb);
                encoder.write_end();
                black_box(ser.into_inner())
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn generate_layer() -> Vec<Run> {
    let mut layer = Vec::new();

    layer.push(Run {
        length: 3840,
        value: 0,
    });

    for _ in 0..PLATFORM.y / 2 {
        layer.extend_from_slice(&[
            Run {
                length: 3840,
                value: 255,
            },
            Run {
                length: 7680,
                value: 0,
            },
        ]);
    }

    layer.push(Run {
        length: 3840,
        value: 0,
    });

    layer
}
