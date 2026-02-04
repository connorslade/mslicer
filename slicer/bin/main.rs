use std::{
    fs::{self, File},
    io::{BufReader, Read, Seek, Write, stdout},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{Context, Ok, Result};
use args::{Args, Model};
use clap::{CommandFactory, FromArgMatches};
use clone_macro::clone;
use image::{ImageReader, RgbaImage};

use common::{
    config::SliceConfig,
    progress::Progress,
    serde::{DynamicSerializer, ReaderDeserializer},
};
use slicer::{format::FormatSliceFile, mesh::Mesh, slicer::Slicer};

mod args;

fn main() -> Result<()> {
    let matches = Args::command().get_matches();
    let args = Args::from_arg_matches(&matches)?;
    let models = Model::from_matches(&matches);

    let extension = (args.output.extension())
        .context("Output file has no extension")?
        .to_string_lossy();
    let slice_config = args.slice_config(&extension)?;
    let mm_to_px = args.mm_to_px();

    let mut meshes = Vec::new();
    for model in models {
        let ext = model.path.extension().unwrap().to_string_lossy();
        let buf = BufReader::new(File::open(&model.path)?);

        let mut mesh = load_mesh(buf, &ext)?;

        mesh.set_scale(model.scale);
        mesh.set_rotation(model.rotation.map(f32::to_radians));

        // Center the model
        let (min, max) = mesh.bounds();
        let mesh_center = (min + max) / 2.0;
        let center = (slice_config.platform_resolution / 2).cast::<f32>();
        mesh.set_position((center - mesh_center.xy()).to_homogeneous() + model.position);

        // Scale the model into printer-space (mm => px)
        mesh.set_scale(model.scale.component_mul(&mm_to_px));

        println!(
            "Loaded `{}`. {{ vert: {}, face: {} }}",
            model.path.file_name().unwrap().to_string_lossy(),
            mesh.vertex_count(),
            mesh.face_count()
        );

        if is_oob(&mesh, &slice_config) {
            println!(" \\ Model extends outsize of print volume and will be cut off.",);
        }

        meshes.push(mesh);
    }

    let slicer = Slicer::new(slice_config.clone(), meshes);
    let progress = slicer.progress();
    let total = slicer.layer_count();

    let now = Instant::now();
    let preview = if let Some(path) = args.preview {
        ImageReader::open(path)?.decode()?.to_rgba8()
    } else {
        RgbaImage::new(290, 290)
    };

    let file = thread::spawn(move || {
        let result = slicer.slice_format();
        FormatSliceFile::from_slice_result(&preview, result)
    });

    let file = monitor_progress(file, progress, |progress| {
        format!(
            "\rLayer: {}/{total}, {:.1}%",
            progress.get_complete(),
            progress.progress() * 100.0
        )
    })?;

    println!();
    let progress = Progress::new();
    let handle = thread::spawn(clone!([progress], move || {
        let mut serializer = DynamicSerializer::new();
        file.serialize(&mut serializer, progress);
        fs::write(args.output, serializer.into_inner()).unwrap();
    }));

    monitor_progress(handle, progress, |progress| {
        format!("\rSaving {:.1}%", progress.progress() * 100.0)
    })?;

    println!("\nDone. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}

fn load_mesh<T: Read + Seek + Send + 'static>(reader: T, format: &str) -> Result<Mesh> {
    let des = ReaderDeserializer::new(reader);
    let mesh = mesh_format::load_mesh(des, format, Progress::new())?;
    Ok(Mesh::new(mesh.verts, mesh.faces))
}

fn is_oob(mesh: &Mesh, slice_config: &SliceConfig) -> bool {
    let (min, max) = mesh.bounds();
    min.x < 0.0
        || min.y < 0.0
        || min.z < 0.0
        || max.x > slice_config.platform_resolution.x as f32
        || max.y > slice_config.platform_resolution.y as f32
        || max.z > slice_config.platform_size.z
}

fn monitor_progress<T>(
    handle: JoinHandle<T>,
    progress: Progress,
    callback: impl Fn(&Progress) -> String,
) -> Result<T> {
    let mut stdout = stdout();
    while !handle.is_finished() {
        thread::sleep(Duration::from_millis(50));
        let msg = callback(&progress);
        stdout.write_all(msg.as_bytes())?;
        stdout.flush()?;
    }

    Ok(handle.join().unwrap())
}
