use anyhow::Result;
use common::{
    progress::Progress,
    serde::{Deserializer, Serializer},
};
use nalgebra::Vector3;

mod obj;
mod stl;
mod util;

#[derive(Debug, Default)]
pub struct Mesh {
    pub verts: Box<[Vector3<f32>]>,
    pub faces: Box<[[u32; 3]]>,
}

#[derive(Clone, Copy)]
pub enum Format {
    Stl,
    Obj,
}

impl Mesh {
    pub fn face_normal(&self, face: usize) -> Vector3<f32> {
        let f = self.faces[face];
        let edge1 = self.verts[f[2] as usize] - self.verts[f[1] as usize];
        let edge2 = self.verts[f[0] as usize] - self.verts[f[1] as usize];
        edge1.cross(&edge2).normalize()
    }

    pub fn vertex_normals(&self) -> Vec<Vector3<f32>> {
        let mut normals = vec![Vector3::zeros(); self.verts.len()];
        for (i, [a, b, c]) in self.faces.iter().enumerate() {
            let normal = self.face_normal(i);
            normals[*a as usize] += normal;
            normals[*b as usize] += normal;
            normals[*c as usize] += normal;
        }

        for normal in normals.iter_mut() {
            *normal = normal.normalize();
        }

        normals
    }
}

impl Format {
    pub const ALL: [Self; 2] = [Self::Stl, Self::Obj];

    pub fn from_extension(ext: &str) -> Option<Self> {
        let format = ext.to_ascii_lowercase();
        Some(match format.as_str() {
            "stl" => Self::Stl,
            "obj" => Self::Obj,
            _ => return None,
        })
    }

    pub fn extension(&self) -> &str {
        match self {
            Format::Stl => "stl",
            Format::Obj => "obj",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Format::Stl => "Stereolithography",
            Format::Obj => "Wavefront",
        }
    }
}

pub fn load_mesh<T: Deserializer + Send>(
    mut des: T,
    format: Format,
    progress: &Progress,
) -> Result<Mesh> {
    let mesh = match format {
        Format::Stl => stl::parse(&mut des, progress),
        Format::Obj => obj::parse(&mut des, progress),
    };

    progress.set_finished();
    mesh
}

pub fn save_mesh<T: Serializer>(ser: &mut T, format: Format, progress: &Progress, mesh: &Mesh) {
    match format {
        Format::Stl => stl::serialize(ser, progress, mesh),
        Format::Obj => obj::serialize(ser, progress, mesh),
    }
    progress.set_finished();
}
