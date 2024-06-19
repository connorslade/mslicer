use std::{fs::File, time::Instant};

use anyhow::Result;
use image::RgbImage;
use imageproc::point::Point;
use mesh::load_mesh;
use nalgebra::{Vector2, Vector3};
use tmp_image::{draw_line_segment_invert_mut, draw_polygon_with_mut};

type Pos = Vector3<f32>;

mod mesh;
mod tmp_image;

struct SliceConfig {
    platform_resolution: Vector2<u32>,
    platform_size: Vector3<f32>,
    slice_height: f32,
}

fn main() -> Result<()> {
    const FILE_PATH: &str = "teapot.stl";

    let slice_config = SliceConfig {
        // platform_resolution: Vector2::new(1920, 1080),
        platform_resolution: Vector2::new(11520, 5121),
        platform_size: Vector3::new(218.88, 122.904, 260.0),
        slice_height: 0.05,
    };

    let mut file = File::open(FILE_PATH)?;
    let mut mesh = load_mesh(&mut file, "stl")?;
    let (min, max) = mesh.minmax_point();

    let real_scale = 100.0;
    mesh.scale = Pos::new(real_scale, real_scale, 1.0);

    let center = slice_config.platform_resolution / 2;
    let mesh_center = (min + max) / 2.0;
    mesh.position = Vector3::new(
        center.x as f32 - mesh_center.x,
        center.y as f32 - mesh_center.y,
        mesh.position.z,
    );

    println!(
        "Loaded mesh. {{ vert: {}, face: {} }}",
        mesh.vertices.len(),
        mesh.faces.len()
    );

    let now = Instant::now();

    let mut height = 0.0;
    let mut i = 0;

    let mut image = RgbImage::new(
        slice_config.platform_resolution.x,
        slice_config.platform_resolution.y,
    );

    let max = mesh.transform(&max);
    while height < max.z {
        let intersections = mesh.intersect_plane(height);
        println!("Height: {}, Intersections: {}", height, intersections.len());

        let mut segments = intersections
            .chunks(2)
            .map(|x| (x[0], x[1]))
            .collect::<Vec<_>>();

        if segments.is_empty() {
            height += slice_config.slice_height;
            i += 1;
            continue;
        }

        fn points_equal(a: &Pos, b: &Pos) -> bool {
            (a.x - b.x).abs() < 0.0001 && (a.y - b.y).abs() < 0.0001
        }

        fn points_equal_int(a: &Pos, b: &Pos) -> bool {
            (a.x as i32 == b.x as i32) && (a.y as i32 == b.y as i32)
        }

        let mut polygons = Vec::new();
        let mut polygon = Vec::new();

        'outer: loop {
            if polygon.is_empty() {
                if segments.is_empty() {
                    break;
                }

                let first = segments.remove(0);
                polygon.push(first.0);
                polygon.push(first.1);
            }

            for j in 0..segments.len() {
                let (a, b) = segments[j];
                let last = polygon.last().unwrap();

                if points_equal(&last, &a) {
                    polygon.push(b);
                    segments.remove(j);
                    continue 'outer;
                } else if points_equal(&last, &b) {
                    polygon.push(a);
                    segments.remove(j);
                    continue 'outer;
                }
            }

            polygons.push(polygon.clone());
            polygon.clear();
        }

        for mut polygon in polygons {
            while !polygon.is_empty() && points_equal(&polygon[0], polygon.last().unwrap()) {
                polygon.pop();
            }

            while points_equal_int(&polygon[0], polygon.last().unwrap()) {
                polygon[0].x -= 1.0;
            }

            if polygon.len() < 3 {
                continue;
            }

            let polygons = polygon
                .into_iter()
                .map(|x| Point::new(x.x as i32, x.y as i32))
                .collect::<Vec<_>>();

            draw_polygon_with_mut(
                &mut image,
                &polygons,
                image::Rgb([255, 255, 255]),
                draw_line_segment_invert_mut,
            );
        }

        let filename = format!("slice_output/{i}.png");
        image.save(filename)?;
        image.fill(0);

        height += slice_config.slice_height;
        i += 1;
    }

    println!("Done. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}
