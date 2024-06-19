use std::{
    fs::{self, File},
    time::Instant,
};

use anyhow::Result;
use common::serde::DynamicSerializer;
use goo_format::{File as GooFile, HeaderInfo, LayerContent, LayerEncoder};
use mesh::load_mesh;
use nalgebra::{Vector2, Vector3};
use ordered_float::OrderedFloat;

type Pos = Vector3<f32>;

mod mesh;

struct SliceConfig {
    platform_resolution: Vector2<u32>,
    platform_size: Vector3<f32>,
    slice_height: f32,

    exposure_config: ExposureConfig,
    first_exposure_config: ExposureConfig,
    first_layers: u32,
}

struct ExposureConfig {
    exposure_time: f32,
    lift_distance: f32,
    lift_speed: f32,
    retract_distance: f32,
    retract_speed: f32,
}

fn main() -> Result<()> {
    const FILE_PATH: &str = "teapot.stl";
    const OUTPUT_PATH: &str = "output.goo";

    let slice_config = SliceConfig {
        // platform_resolution: Vector2::new(1920, 1080),
        platform_resolution: Vector2::new(11520, 5121),
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

    let max = mesh.transform(&max);
    let layers = (max.z / slice_config.slice_height).ceil() as u32;

    let layers = (0..layers)
        .map(|layer| {
            let height = layer as f32 * slice_config.slice_height;

            let intersections = mesh.intersect_plane(height);
            println!("Height: {}, Intersections: {}", height, intersections.len());

            let segments = intersections
                .chunks(2)
                .map(|x| (x[0], x[1]))
                .collect::<Vec<_>>();

            let mut out = Vec::new();
            for y in 0..slice_config.platform_resolution.y {
                let mut intersections = segments
                    .iter()
                    .filter_map(|(a, b)| {
                        let y = y as f32;
                        if a.y <= y && b.y >= y {
                            let t = (y - a.y) / (b.y - a.y);
                            let x = a.x + t * (b.x - a.x);
                            Some(x)
                        } else if b.y <= y && a.y >= y {
                            let t = (y - b.y) / (a.y - b.y);
                            let x = b.x + t * (a.x - b.x);
                            Some(x)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                intersections.sort_by_key(|&x| OrderedFloat(x));
                intersections.dedup();

                for span in intersections.chunks_exact(2) {
                    let y_offset = (slice_config.platform_resolution.x * y) as u64;
                    out.push((y_offset + span[0] as u64, y_offset + span[1] as u64));
                }
            }

            let mut encoder = LayerEncoder::new();

            let mut last = 0;
            for (start, end) in out {
                if start > last {
                    encoder.add_run(start - last, 0);
                }

                encoder.add_run(end - start, 1);
                last = end;
            }

            let (data, checksum) = encoder.finish();
            let layer_exposure = if layer < slice_config.first_layers {
                &slice_config.first_exposure_config
            } else {
                &slice_config.exposure_config
            };

            LayerContent {
                data,
                checksum,
                layer_position_z: slice_config.slice_height * layer as f32,

                layer_exposure_time: layer_exposure.exposure_time,
                lift_distance: layer_exposure.lift_distance,
                lift_speed: layer_exposure.lift_speed,
                retract_distance: layer_exposure.retract_distance,
                retract_speed: layer_exposure.retract_speed,
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();

    let layer_time = slice_config.exposure_config.exposure_time
        + slice_config.exposure_config.lift_distance / slice_config.exposure_config.lift_speed;
    let bottom_layer_time = slice_config.first_exposure_config.exposure_time
        + slice_config.first_exposure_config.lift_distance
            / slice_config.first_exposure_config.lift_speed;
    let total_time = (layers.len() as u32 - slice_config.first_layers) as f32 * layer_time
        + slice_config.first_layers as f32 * bottom_layer_time;

    let goo = GooFile::new(
        HeaderInfo {
            layer_count: layers.len() as u32,

            printing_time: total_time as u32,
            layer_thickness: slice_config.slice_height,
            bottom_layers: slice_config.first_layers,
            transition_layers: slice_config.first_layers as u16 + 1,

            exposure_time: slice_config.exposure_config.exposure_time,
            lift_distance: slice_config.exposure_config.lift_distance,
            lift_speed: slice_config.exposure_config.lift_speed,
            retract_distance: slice_config.exposure_config.retract_distance,
            retract_speed: slice_config.exposure_config.retract_speed,

            bottom_exposure_time: slice_config.first_exposure_config.exposure_time,
            bottom_lift_distance: slice_config.first_exposure_config.lift_distance,
            bottom_lift_speed: slice_config.first_exposure_config.lift_speed,
            bottom_retract_distance: slice_config.first_exposure_config.retract_distance,
            bottom_retract_speed: slice_config.first_exposure_config.retract_speed,

            ..Default::default()
        },
        layers,
    );

    let mut serializer = DynamicSerializer::new();
    goo.serialize(&mut serializer);
    fs::write(OUTPUT_PATH, serializer.into_inner())?;

    println!("Done. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}

impl Default for ExposureConfig {
    fn default() -> Self {
        Self {
            exposure_time: 3.0,
            lift_distance: 5.0,
            lift_speed: 65.0,
            retract_distance: 5.0,
            retract_speed: 150.0,
        }
    }
}
