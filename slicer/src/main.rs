use std::{
    fs::{self, File},
    io::{stdout, BufReader, Write},
    path::PathBuf,
    thread,
    time::Instant,
};

use anyhow::Result;
use clap::Parser;
use log::{debug, warn};
use nalgebra::{Vector2, Vector3};

use common::{
    config::{ExposureConfig, SliceConfig},
    format::Format,
    serde::DynamicSerializer,
};
use goo_format::{File as GooFile, LayerEncoder};
use slicer::{mesh::load_mesh, slicer::Slicer, Pos};

#[derive(Parser)]
struct Args {
    /// Path to the .stl file
    input_file: PathBuf,
    /// Path to the .goo file
    output_file: PathBuf,

    /// Rotate the model (in degrees, about the z axis) before slicing it
    #[clap(long)]
    rotate_xy: Option<f32>,
    /// Rotate the model (in degrees, about the y axis) before slicing it
    #[clap(long)]
    rotate_xz: Option<f32>,
    /// Rotate the model (in degrees, about the x axis) before slicing it
    #[clap(long)]
    rotate_yz: Option<f32>,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

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

    let file = File::open(args.input_file)?;
    let mut buf = BufReader::new(file);
    let mut mesh = load_mesh(&mut buf, "stl")?;

    let mut rotate = mesh.rotation();
    if let Some(r) = args.rotate_xy {
        rotate.z += r.to_radians();
    }
    if let Some(r) = args.rotate_xz {
        rotate.y += r.to_radians();
    }
    if let Some(r) = args.rotate_yz {
        rotate.x += r.to_radians();
    }
    mesh.set_rotation(rotate);

    let (min, max) = mesh.bounds();

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

    let mesh_size = max - min;
    debug!("mesh_size: {}", mesh_size);
    debug!("platform_size: {}", slice_config.platform_size);

    if mesh_size.x > slice_config.platform_size.x || mesh_size.y > slice_config.platform_size.y || mesh_size.z > slice_config.platform_size.z {
        warn!("WARNING: model bounds ({}) exceeds printer bounds ({}); print may be truncated", mesh_size, slice_config.platform_size);
    }

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
    fs::write(args.output_file, serializer.into_inner())?;

    println!("\nDone. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}
