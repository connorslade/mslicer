use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Seek, Write, stdout},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Ok, Result};
use args::{Args, Model};
use clap::{CommandFactory, FromArgMatches};
use clone_macro::clone;
use image::{ImageReader, RgbaImage};

use common::{
    progress::Progress,
    serde::{DynamicSerializer, ReaderDeserializer},
    slice::{SliceConfig, format::RasterFormat},
    units::Milimeter,
};
use slicer::{
    mesh::Mesh,
    slicer::{Slicer, SlicerModel},
    util::export_raster,
};

mod args;

/// Terminal escape sequence to clear the line and move the cursor to the start.
const RESET_LINE: &str = "\u{001b}[0G\u{001b}[K";

fn main() -> Result<()> {
    let matches = Args::command().get_matches();
    let args = Args::from_arg_matches(&matches)?;
    let models = Model::from_matches(&matches);

    let extension = (args.output.extension())
        .context("Output file has no extension")?
        .to_string_lossy();
    let format = RasterFormat::from_extension(&extension).context("Unknown output format")?;

    let slice_config = args.slice_config()?;
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

        meshes.push(SlicerModel {
            mesh,
            exposure: 255,
        });
    }

    let slicer = Slicer::new(slice_config.clone(), meshes);
    let slicing_progress = slicer.progress();
    let encoding_progress = Progress::new();
    let total = slicer.layer_count();
    encoding_progress.set_total(total as u64);

    let now = Instant::now();
    let preview = if let Some(path) = args.preview {
        ImageReader::open(path)?.decode()?.to_rgba8()
    } else {
        RgbaImage::new(290, 290)
    };

    let handle = thread::spawn(clone!([{ encoding_progress } as p], move || {
        let layers = slicer.slice_raster();
        let voxels = (layers.iter())
            .flat_map(|x| x.data.iter().filter(|x| x.value != 0).map(|x| x.length))
            .sum::<u64>();
        export_raster(&p, &slicer.slice_config, layers, voxels, format)
    }));

    if !args.quiet {
        monitor_progress(slicing_progress, |p, n| {
            format!("[1/3] Slicing: {n}/{total}, {p:.1}%")
        })?;
        monitor_progress(encoding_progress, |p, n| {
            format!("[2/3] Encoding: {n}/{total}, {p:.1}%")
        })?;
    }

    let mut file = handle.join().unwrap();
    file.set_preview(&preview);

    let progress = Progress::new();
    let handle = thread::spawn(clone!([progress], move || {
        let mut serializer = DynamicSerializer::new();
        file.serialize(&mut serializer, &progress);
        fs::write(args.output, serializer.into_inner()).unwrap();
    }));

    if !args.quiet {
        monitor_progress(progress, |p, _n| format!("[3/3] Saving: {p:.1}%"))?;
    }
    handle.join().unwrap();

    println!("{RESET_LINE}Finished in {:.1?}", now.elapsed());
    Ok(())
}

fn load_mesh<T: Read + Seek + Send + 'static>(reader: T, format: &str) -> Result<Mesh> {
    let des = ReaderDeserializer::new(reader);
    let format = mesh_format::Format::from_extension(format).context("Unknown mesh extension.")?;
    let mesh = mesh_format::load_mesh(des, format, &Progress::new())?;
    Ok(Mesh::new_boxed(mesh.verts, mesh.faces))
}

fn is_oob(mesh: &Mesh, slice_config: &SliceConfig) -> bool {
    let (min, max) = mesh.bounds();
    min.x < 0.0
        || min.y < 0.0
        || min.z < 0.0
        || max.x > slice_config.platform_resolution.x as f32
        || max.y > slice_config.platform_resolution.y as f32
        || max.z > slice_config.platform_size.z.get::<Milimeter>()
}

fn monitor_progress(progress: Progress, callback: impl Fn(f32, u64) -> String) -> Result<()> {
    let mut stdout = BufWriter::new(stdout());
    while !progress.complete() {
        thread::sleep(Duration::from_millis(50));
        let msg = callback(progress.progress() * 100.0, progress.get_complete());
        stdout.write_all(RESET_LINE.as_bytes())?;
        stdout.write_all(msg.as_bytes())?;
        stdout.flush()?;
    }

    Ok(())
}
