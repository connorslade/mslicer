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
        let height = self.inv_transform(&Pos::new(0.0, 0.0, height)).z;
        let mut out = Vec::new();

        for face in &self.faces {
            let v0 = self.vertices[face[0] as usize];
            let v1 = self.vertices[face[1] as usize];
            let v2 = self.vertices[face[2] as usize];

            let (a, b, c) = (v0.z - height, v1.z - height, v2.z - height);
            let (a_pos, b_pos, c_pos) = (a > 0.0, b > 0.0, c > 0.0);

            let mut push_intersection = |a: f32, b: f32, v0: Pos, v1: Pos| {
                let t = a / (a - b);
                let intersection = v0 + t * (v1 - v0);
                out.push(self.transform(&intersection));
            };

            (a_pos ^ b_pos).then(|| push_intersection(a, b, v0, v1));
            (b_pos ^ c_pos).then(|| push_intersection(b, c, v1, v2));
            (c_pos ^ a_pos).then(|| push_intersection(c, a, v2, v0));
        }

        out
    }

    pub fn transform(&self, pos: &Pos) -> Pos {
        Pos::new(
            pos.x * self.scale.x,
            pos.y * self.scale.y,
            pos.z * self.scale.z,
        ) + self.position
    }

    pub fn inv_transform(&self, pos: &Pos) -> Pos {
        Pos::new(
            (pos.x - self.position.x) / self.scale.x,
            (pos.y - self.position.y) / self.scale.y,
            (pos.z - self.position.z) / self.scale.z,
        )
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
