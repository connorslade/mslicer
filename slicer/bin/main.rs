use std::{
    fs::{self, File},
    io::{stdout, BufReader, Write},
    thread,
    time::Instant,
};

use anyhow::Result;
use args::{Args, CliMesh};
use clap::{CommandFactory, FromArgMatches};
use nalgebra::Vector3;

use common::serde::DynamicSerializer;
use goo_format::{File as GooFile, LayerEncoder};
use slicer::{mesh::load_mesh, slicer::Slicer};

mod args;

fn main() -> Result<()> {
    let matches = Args::command().get_matches();
    let args = Args::from_arg_matches(&matches)?;
    let slice_config = args.slice_config();
    let cli_meshes = CliMesh::from_matches(&matches);

    let mut meshes = Vec::new();

    for cli_mesh in cli_meshes {
        let ext = cli_mesh.path.extension().unwrap().to_string_lossy();
        let file = File::open(&cli_mesh.path)?;

        let mut buf = BufReader::new(file);
        let mut mesh = load_mesh(&mut buf, &ext)?;

        // Scale the model into printer-space (mm => px)
        mesh.set_scale(cli_mesh.scale.component_div(&Vector3::new(
            slice_config.platform_size.x * slice_config.platform_resolution.x as f32,
            slice_config.platform_size.y * slice_config.platform_resolution.y as f32,
            1.0,
        )));

        mesh.set_rotation(cli_mesh.rotation);

        // Center the model
        let (min, max) = mesh.bounds();
        let center = slice_config.platform_resolution / 2;
        let mesh_center = (min + max) / 2.0;
        mesh.set_position(
            Vector3::new(
                center.x as f32 - mesh_center.x,
                center.y as f32 - mesh_center.y,
                mesh.position().z - 0.05,
            ) + cli_mesh.position,
        );

        println!(
            "Loaded `{}`. {{ vert: {}, face: {} }}",
            cli_mesh.path.file_name().unwrap().to_string_lossy(),
            mesh.vertex_count(),
            mesh.face_count()
        );

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
    let mut serializer = DynamicSerializer::new();
    goo.join().unwrap().serialize(&mut serializer);
    fs::write(args.output, serializer.into_inner())?;

    println!("\nDone. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}
