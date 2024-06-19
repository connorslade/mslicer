use std::io::{Read, Seek};

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::Pos;

pub struct Mesh {
    pub vertices: Vec<Pos>,
    pub faces: Vec<[usize; 3]>,

    pub position: Pos,
    pub scale: Pos,
}

impl Mesh {
    fn center_vertices(mut self) -> Self {
        let (min, max) = self.minmax_point();

        let center = (min + max) / 2.0;
        let center = Pos::new(center.x, center.y, min.z);

        for v in self.vertices.iter_mut() {
            *v -= center;
        }

        self
    }

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

    pub fn intersect_plane(&self, height: f32) -> Vec<Pos> {
        self.faces
            .par_iter()
            .flat_map(|face| {
                let v0 = self.transform(&self.vertices[face[0]]);
                let v1 = self.transform(&self.vertices[face[1]]);
                let v2 = self.transform(&self.vertices[face[2]]);

                let dot0 = v0.z - height;
                let dot1 = v1.z - height;
                let dot2 = v2.z - height;

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

    pub fn transform(&self, pos: &Pos) -> Pos {
        Pos::new(
            pos.x * self.scale.x,
            pos.y * self.scale.y,
            pos.z * self.scale.z,
        ) + self.position
    }
}

pub fn load_mesh<T: Read + Seek>(reader: &mut T, format: &str) -> Result<Mesh> {
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
            }
            .center_vertices())
        }
        _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
    }
}
