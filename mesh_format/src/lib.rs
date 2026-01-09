use std::thread::{self, JoinHandle};

use anyhow::Result;
use clone_macro::clone;
use common::{progress::Progress, serde::Deserializer};
use nalgebra::Vector3;

mod obj;
mod stl;
mod util;

#[derive(Debug, Default)]
pub struct Mesh {
    pub verts: Vec<Vector3<f32>>,
    pub faces: Vec<[u32; 3]>,
}

pub fn load_mesh<T: Deserializer + Send + 'static>(
    mut des: T,
    format: &str,
) -> (Progress, JoinHandle<Result<Mesh>>) {
    let progress = Progress::new();

    let format = format.to_ascii_lowercase();
    let join = thread::spawn(clone!([progress], move || {
        let mesh = match format.as_str() {
            "stl" => stl::parse(&mut des, progress.clone()),
            "obj" => obj::parse(&mut des, progress.clone()),
            _ => panic!("Unsupported format: {}", format),
        };

        progress.set_finished();
        mesh
    }));

    (progress, join)
}
