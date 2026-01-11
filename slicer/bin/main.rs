use std::{
    fs::{self, File},
    io::{BufReader, Write, stdout},
    thread,
    time::Instant,
};

use anyhow::Result;
use args::{Args, Model};
use clap::{CommandFactory, FromArgMatches};
use image::{ImageReader, imageops::FilterType};

use common::serde::DynamicSerializer;
use goo_format::{File as GooFile, LayerEncoder, PreviewImage};
use slicer::{mesh::load_mesh, slicer::Slicer};

mod args;

fn main() -> Result<()> {
    let matches = Args::command().get_matches();
    let args = Args::from_arg_matches(&matches)?;
    let models = Model::from_matches(&matches);

    let slice_config = args.slice_config();
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
        let center = (slice_config.platform_resolution / 2).map(|x| x as f32);
        mesh.set_position((center - mesh_center.xy()).to_homogeneous() + model.position);

        // Scale the model into printer-space (mm => px)
        mesh.set_scale(model.scale.component_mul(&mm_to_px));

        println!(
            "Loaded `{}`. {{ vert: {}, face: {} }}",
            model.path.file_name().unwrap().to_string_lossy(),
            mesh.vertex_count(),
            mesh.face_count()
        );

        let (min, max) = mesh.bounds();
        if min.x < 0.0
            || min.y < 0.0
            || min.z < 0.0
            || max.x > slice_config.platform_resolution.x as f32
            || max.y > slice_config.platform_resolution.y as f32
            || max.z > slice_config.platform_size.z
        {
            println!(" \\ Model extends outsize of print volume and will be cut off.",);
        }

        meshes.push(mesh);
    }

    // Actually slice it on another thread (the slicing is multithreaded)
    let now = Instant::now();

    let slicer = Slicer::new(slice_config.clone(), meshes);
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
    let mut goo = goo.join().unwrap();

    if let Some(path) = args.preview {
        let image = ImageReader::open(path)?.decode()?.to_rgba8();
        goo.header.small_preview = PreviewImage::from_image_scaled(&image, FilterType::Triangle);
        goo.header.big_preview = PreviewImage::from_image_scaled(&image, FilterType::Triangle);
    }

    let mut serializer = DynamicSerializer::new();
    goo.serialize(&mut serializer);
    fs::write(args.output, serializer.into_inner())?;

    println!("\nDone. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}
