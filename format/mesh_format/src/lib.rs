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

pub enum Format {
    Stl,
    Obj,
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
