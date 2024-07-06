use std::{
    io::{BufRead, Seek},
    sync::Arc,
};

use anyhow::Result;
use nalgebra::Matrix4;
use obj::{Obj, Position};

use crate::Pos;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Arc<Box<[Pos]>>,
    pub faces: Arc<Box<[[u32; 3]]>>,

    transformation_matrix: Matrix4<f32>,
    inv_transformation_matrix: Matrix4<f32>,

    position: Pos,
    scale: Pos,
    rotation: Pos,
}

impl Mesh {
    pub fn new(mut vertices: Vec<Pos>, faces: Vec<[u32; 3]>) -> Self {
        center_vertices(&mut vertices);

        Self {
            vertices: Arc::new(vertices.into_boxed_slice()),
            faces: Arc::new(faces.into_boxed_slice()),
            ..Default::default()
        }
    }

    pub fn intersect_plane(&self, height: f32) -> Vec<Pos> {
        let mut out = Vec::new();

        for face in self.faces.iter() {
            // todo: just transform the plane with the inverse transformation matrix
            let v0 = self.transform(&self.vertices[face[0] as usize]);
            let v1 = self.transform(&self.vertices[face[1] as usize]);
            let v2 = self.transform(&self.vertices[face[2] as usize]);

            let (a, b, c) = (v0.z - height, v1.z - height, v2.z - height);
            let (a_pos, b_pos, c_pos) = (a > 0.0, b > 0.0, c > 0.0);

            let mut push_intersection = |a: f32, b: f32, v0: Pos, v1: Pos| {
                let t = a / (a - b);
                let intersection = v0 + t * (v1 - v0);
                out.push(intersection);
            };

            (a_pos ^ b_pos).then(|| push_intersection(a, b, v0, v1));
            (b_pos ^ c_pos).then(|| push_intersection(b, c, v1, v2));
            (c_pos ^ a_pos).then(|| push_intersection(c, a, v2, v0));
        }

        out
    }

    pub fn update_transformation_matrix(&mut self) {
        let scale = Matrix4::new_nonuniform_scaling(&self.scale);
        let rotation =
            Matrix4::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let translation = Matrix4::new_translation(&self.position);

        self.transformation_matrix = translation * rotation * scale;
        self.inv_transformation_matrix = self.transformation_matrix.try_inverse().unwrap();
    }

    pub fn transform(&self, pos: &Pos) -> Pos {
        (self.transformation_matrix * pos.push(1.0)).xyz()
    }

    pub fn inv_transform(&self, pos: &Pos) -> Pos {
        (self.inv_transformation_matrix * pos.push(1.0)).xyz()
    }

    pub fn minmax_point(&self) -> (Pos, Pos) {
        minmax_vertices(&self.vertices, &self.transformation_matrix)
    }
}

impl Mesh {
    pub fn transformation_matrix(&self) -> &Matrix4<f32> {
        &self.transformation_matrix
    }

    pub fn set_position(&mut self, pos: Pos) {
        self.position = pos;
        self.update_transformation_matrix();
    }

    pub fn set_position_unchecked(&mut self, pos: Pos) {
        self.position = pos;
    }

    pub fn position(&self) -> Pos {
        self.position
    }

    pub fn set_scale(&mut self, scale: Pos) {
        self.scale = scale;
        self.update_transformation_matrix();
    }

    pub fn set_scale_unchecked(&mut self, scale: Pos) {
        self.scale = scale;
    }

    pub fn scale(&self) -> Pos {
        self.scale
    }

    pub fn set_rotation(&mut self, rotation: Pos) {
        self.rotation = rotation;
        self.update_transformation_matrix();
    }

    pub fn set_rotation_unchecked(&mut self, rotation: Pos) {
        self.rotation = rotation;
    }

    pub fn rotation(&self) -> Pos {
        self.rotation
    }
}

pub fn load_mesh<T: BufRead + Seek>(reader: &mut T, format: &str) -> Result<Mesh> {
    Ok(match format {
        "stl" => {
            let model = stl_io::read_stl(reader)?;
            Mesh::new(
                model
                    .vertices
                    .iter()
                    .map(|v| Pos::new(v[0], v[1], v[2]))
                    .collect(),
                model
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
            )
        }
        "obj" | "mtl" => {
            let model: Obj<Position> = obj::load_obj(reader)?;
            Mesh::new(
                model
                    .vertices
                    .iter()
                    .map(|v| Pos::new(v.position[0], v.position[1], v.position[2]))
                    .collect(),
                model
                    .indices
                    .chunks_exact(3)
                    .map(|i| [i[0] as u32, i[1] as u32, i[2] as u32])
                    .collect(),
            )
        }
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", format)),
    })
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            faces: Default::default(),

            transformation_matrix: Matrix4::identity(),
            inv_transformation_matrix: Matrix4::identity(),

            position: Pos::new(0.0, 0.0, 0.0),
            scale: Pos::new(1.0, 1.0, 1.0),
            rotation: Pos::new(0.0, 0.0, 0.0),
        }
    }
}

fn minmax_vertices(vertices: &[Pos], transform: &Matrix4<f32>) -> (Pos, Pos) {
    vertices.iter().fold(
        (
            Pos::new(f32::MAX, f32::MAX, f32::MAX),
            Pos::new(f32::MIN, f32::MIN, f32::MIN),
        ),
        |(min, max), v| {
            let v = transform * v.push(1.0);
            (
                Pos::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z)),
                Pos::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z)),
            )
        },
    )
}

fn center_vertices(vertices: &mut [Pos]) {
    let (min, max) = minmax_vertices(vertices, &Matrix4::identity());

    let center = (min + max) / 2.0;
    let center = Pos::new(center.x, center.y, min.z);
    vertices.iter_mut().for_each(|v| *v -= center);
}
