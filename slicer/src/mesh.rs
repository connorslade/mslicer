use std::io::{Read, Seek};

use anyhow::Result;

use crate::Pos;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Pos>,
    pub faces: Vec<[u32; 3]>,

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
            .iter()
            .flat_map(|face| {
                let v0 = self.transform(&self.vertices[face[0] as usize]);
                let v1 = self.transform(&self.vertices[face[1] as usize]);
                let v2 = self.transform(&self.vertices[face[2] as usize]);

                let a = v0.z - height;
                let b = v1.z - height;
                let c = v2.z - height;

                let a_pos = a > 0.0;
                let b_pos = b > 0.0;
                let c_pos = c > 0.0;

                let mut result = [Pos::zeros(); 2];
                let mut index = 0;

                if a_pos ^ b_pos {
                    let t = a / (a - b);
                    let intersection = v0 + t * (v1 - v0);
                    result[index] = intersection;
                    index += 1;
                }

                if b_pos ^ c_pos {
                    let t = b / (b - c);
                    let intersection = v1 + t * (v2 - v1);
                    result[index] = intersection;
                    index += 1;
                }

                if c_pos ^ a_pos {
                    let t = c / (c - a);
                    let intersection = v2 + t * (v0 - v2);
                    result[index] = intersection;
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
            let model = stl_io::read_stl(reader)?;
            Ok(Mesh {
                vertices: model
                    .vertices
                    .iter()
                    .map(|v| Pos::new(v[0], v[1], v[2]))
                    .collect(),
                faces: model
                    .faces
                    .iter()
                    .map(|f| {
                        [
                            f.vertices[0] as u32,
                            f.vertices[1] as u32,
                            f.vertices[2] as u32,
                        ]
                    })
                    .collect(),

                position: Pos::new(0.0, 0.0, 0.0),
                scale: Pos::new(1.0, 1.0, 1.0),
            }
            .center_vertices())
        }
        _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
    }
}
