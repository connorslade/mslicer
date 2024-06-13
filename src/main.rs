use std::{
    fs::File,
    io::{Read, Seek},
};

use anyhow::Result;
use image::RgbImage;
use nalgebra::Vector3;

type Pos = Vector3<f32>;

struct Mesh {
    vertices: Vec<Pos>,
    faces: Vec<[usize; 3]>,
}

fn main() -> Result<()> {
    const FILE_PATH: &str = "teapot.stl";

    let mut file = File::open(FILE_PATH)?;
    let mesh = load_mesh(&mut file, "stl")?;

    println!(
        "Loaded mesh. {{ vert: {}, face: {} }}",
        mesh.vertices.len(),
        mesh.faces.len()
    );

    let (min, max) = mesh.minmax_point();
    let slice_height = 0.1;

    let mut height = 0.0;
    let mut i = 0;

    while height < max.z {
        let plane_normal = Pos::new(0.0, 0.0, 1.0);
        let plane_point = Pos::new(0.0, 0.0, height);

        let intersections = mesh.intersect_plane(&plane_normal, &plane_point);
        println!("Height: {}, Intersections: {}", height, intersections.len());

        let mut image = RgbImage::new(1920, 1080);
        for intersection in intersections.chunks(2) {
            let x1 = ((intersection[0].x - min.x) / (max.x - min.x)) * image.width() as f32;
            let y1 = ((intersection[0].y - min.y) / (max.y - min.y)) * image.height() as f32;

            let x2 = ((intersection[1].x - min.x) / (max.x - min.x)) * image.width() as f32;
            let y2 = ((intersection[1].y - min.y) / (max.y - min.y)) * image.height() as f32;

            imageproc::drawing::draw_line_segment_mut(
                &mut image,
                (x1 as f32, y1 as f32),
                (x2 as f32, y2 as f32),
                image::Rgb([255, 255, 255]),
            );
        }

        let filename = format!("output/{i}.png");
        image.save(filename)?;

        height += slice_height;
        i += 1;
    }

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
        let mut result = Vec::new();

        for face in &self.faces {
            for (v0, v1) in [
                (self.vertices[face[0]], self.vertices[face[1]]),
                (self.vertices[face[1]], self.vertices[face[2]]),
                (self.vertices[face[2]], self.vertices[face[0]]),
            ] {
                let d0 = v0 - point;
                let d1 = v1 - point;

                let dot0 = normal.dot(&d0);
                let dot1 = normal.dot(&d1);

                if dot0 * dot1 < 0.0 {
                    let t = dot0 / (dot0 - dot1);
                    let intersection = v0 + t * (v1 - v0);
                    result.push(intersection);
                }
            }
        }

        result
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
            })
        }
        _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
    }
}
