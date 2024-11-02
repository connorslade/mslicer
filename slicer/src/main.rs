use std::{
    fs::{self, File},
    io::{stdout, BufReader, Write},
    thread,
    time::Instant,
};

use anyhow::Result;
use nalgebra::{Vector2, Vector3};

use common::{
    config::{ExposureConfig, SliceConfig},
    format::Format,
    serde::DynamicSerializer,
};
use goo_format::{File as GooFile, LayerEncoder};
use slicer::{mesh::load_mesh, slicer::Slicer, Pos};

fn main() -> Result<()> {
    const FILE_PATH: &str = "teapot.stl";
    const OUTPUT_PATH: &str = "output.goo";

    let slice_config = SliceConfig {
        format: Format::Goo,

        platform_resolution: Vector2::new(11_520, 5_120),
        platform_size: Vector3::new(218.88, 122.904, 260.0),
        slice_height: 0.05,

        exposure_config: ExposureConfig {
            exposure_time: 3.0,
            ..Default::default()
        },
        first_exposure_config: ExposureConfig {
            exposure_time: 50.0,
            ..Default::default()
        },
        first_layers: 10,
        transition_layers: 10,
    };

    let file = File::open(FILE_PATH)?;
    let mut buf = BufReader::new(file);
    let mut mesh = load_mesh(&mut buf, "stl")?;
    let (min, max) = mesh.minmax_point();

    // Scale the model into printer-space (mm => px)
    let real_scale = 1.0;
    mesh.set_scale(Pos::new(
        real_scale / slice_config.platform_size.x * slice_config.platform_resolution.x as f32,
        real_scale / slice_config.platform_size.y * slice_config.platform_resolution.y as f32,
        real_scale,
    ));

    // Center the model
    let center = slice_config.platform_resolution / 2;
    let mesh_center = (min + max) / 2.0;
    mesh.set_position(Vector3::new(
        center.x as f32 - mesh_center.x,
        center.y as f32 - mesh_center.y,
        mesh.position().z - 0.05,
    ));

    println!(
        "Loaded mesh. {{ vert: {}, face: {} }}",
        mesh.vertex_count(),
        mesh.face_count()
    );

    // Actually slice it on another thread (the slicing is multithreaded)
    let now = Instant::now();

    let slicer = Slicer::new(slice_config.clone(), vec![mesh]);
    let progress = slicer.progress();

    let goo = thread::spawn(move || GooFile::from_slice_result(slicer.slice::<LayerEncoder>()));

    let mut completed = 0;
    while completed < progress.total() {
        completed = progress.wait();
        print!(
            "\rLayer: {}/{}, {:.1}%",
            completed,
            progress.total(),
            completed as f32 / progress.total() as f32 * 100.0
        );
        stdout().flush()?;
    }

    // Once slicing is complete write to a .goo file
    let mut serializer = DynamicSerializer::new();
    goo.join().unwrap().serialize(&mut serializer);
    fs::write(OUTPUT_PATH, serializer.into_inner())?;

    println!("\nDone. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}
