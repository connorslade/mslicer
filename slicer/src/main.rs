use std::{
    fs::File,
    io::{Read, Seek},
    time::Instant,
};

use anyhow::Result;
use image::RgbImage;
use nalgebra::{Vector2, Vector3};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

type Pos = Vector3<f32>;

struct Mesh {
    vertices: Vec<Pos>,
    faces: Vec<[usize; 3]>,

    position: Pos,
    scale: Pos,
}

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

    let real_scale = 15.0;
    mesh.scale = Pos::new(real_scale, real_scale, real_scale);

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

    let plane_normal = Pos::new(0.0, 0.0, 1.0);
    let mut image = RgbImage::new(
        slice_config.platform_resolution.x,
        slice_config.platform_resolution.y,
    );

    while height < max.z {
        let plane_point = Pos::new(0.0, 0.0, height);

        let intersections = mesh.intersect_plane(&plane_normal, &plane_point);
        println!("Height: {}, Intersections: {}", height, intersections.len());

        for intersection in intersections.chunks(2) {
            let a = intersection[0];
            let b = intersection[1];

            imageproc::drawing::draw_line_segment_mut(
                &mut image,
                (a.x, a.y),
                (b.x, b.y),
                image::Rgb([255, 255, 255]),
            );
        }

        let filename = format!("output/{i}.png");
        image.save(filename)?;
        image.fill(0);

        height += slice_config.slice_height;
        i += 1;
    }

    println!("Done. Elapsed: {:.1}s", now.elapsed().as_secs_f32());

    Ok(())
}

impl Mesh {
    pub fn minmax_point(&self) -> (Pos, Pos) {
        self.vertices.iter().fold(
            (
                Pos::new(f32::MAX, f32::MAX, f32::MAX),
                Pos::new(f32::MIN, f32::MIN, f32::MIN),
            ),
            |(min, max), v| {
                (
                    Pos::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z)),
                    Pos::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z)),
                )
            },
        )
    }

    pub fn intersect_plane(&self, normal: &Pos, point: &Pos) -> Vec<Pos> {
        self.faces
            .par_iter()
            .flat_map(|face| {
                let v0 = self.transform(&self.vertices[face[0]]);
                let v1 = self.transform(&self.vertices[face[1]]);
                let v2 = self.transform(&self.vertices[face[2]]);

                let d0 = v0 - point;
                let d1 = v1 - point;
                let d2 = v2 - point;

                let dot0 = normal.dot(&d0);
                let dot1 = normal.dot(&d1);
                let dot2 = normal.dot(&d2);

                let mut result = Vec::new();

                if dot0 * dot1 < 0.0 {
                    let t = dot0 / (dot0 - dot1);
                    let intersection = v0 + t * (v1 - v0);
                    result.push(intersection);
                }

                if dot1 * dot2 < 0.0 {
                    let t = dot1 / (dot1 - dot2);
                    let intersection = v1 + t * (v2 - v1);
                    result.push(intersection);
                }

                if dot2 * dot0 < 0.0 {
                    let t = dot2 / (dot2 - dot0);
                    let intersection = v2 + t * (v0 - v2);
                    result.push(intersection);
                }

                result
            })
            .collect()
    }

    fn transform(&self, pos: &Pos) -> Pos {
        Pos::new(
            pos.x * self.scale.x,
            pos.y * self.scale.y,
            pos.z * self.scale.z,
        ) + self.position
    }
}

fn load_mesh<T: Read + Seek>(reader: &mut T, format: &str) -> Result<Mesh> {
    match format {
        "stl" => {
            let modal = stl_io::read_stl(reader)?;
            Ok(Mesh {
                vertices: modal
                    .vertices
                    .iter()
                    .map(|v| Pos::new(v[0], v[1], v[2]))
                    .collect(),
                faces: modal
                    .faces
                    .iter()
                    .map(|f| [f.vertices[0], f.vertices[1], f.vertices[2]])
                    .collect(),

                position: Pos::new(0.0, 0.0, 0.0),
                scale: Pos::new(1.0, 1.0, 1.0),
            })
        }
        _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
    }
}
