use std::{
    fs::{self, File},
    time::Instant,
};

use anyhow::Result;
use common::serde::DynamicSerializer;
use nalgebra::{Vector2, Vector3};

use slicer::{
    mesh::load_mesh,
    slicer::{slice_goo, ExposureConfig, SliceConfig},
    Pos,
};

fn main() -> Result<()> {
    const FILE_PATH: &str = "teapot.stl";
    const OUTPUT_PATH: &str = "output.goo";

    let slice_config = SliceConfig {
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
    };

    let mut file = File::open(FILE_PATH)?;
    let mut mesh = load_mesh(&mut file, "stl")?;
    let (min, max) = mesh.minmax_point();

    let real_scale = 1.0;
    mesh.scale = Pos::new(
        real_scale / slice_config.platform_size.x * slice_config.platform_resolution.x as f32,
        real_scale / slice_config.platform_size.y * slice_config.platform_resolution.y as f32,
        real_scale,
    );

    let center = slice_config.platform_resolution / 2;
    let mesh_center = (min + max) / 2.0;
    mesh.position = Vector3::new(
        center.x as f32 - mesh_center.x,
        center.y as f32 - mesh_center.y,
        mesh.position.z - 0.05,
    );

    println!(
        "Loaded mesh. {{ vert: {}, face: {} }}",
        mesh.vertices.len(),
        mesh.faces.len()
    );

    let now = Instant::now();

    let goo = slice_goo(&slice_config, &mesh, |layer, layers| {
        print!(
            "\rLayer: {}/{layers} ({:.1}%)",
            layer + 1,
            (layer as f32 + 1.0) / layers as f32 * 100.0
        );
    });

    let mut serializer = DynamicSerializer::new();
    goo.serialize(&mut serializer);
    fs::write(OUTPUT_PATH, serializer.into_inner())?;

    println!("\nDone. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}
