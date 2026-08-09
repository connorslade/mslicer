//! Marching cubes implementation is modified from my wave-sim-3d project.
//! https://github.com/connorslade/wave-sim-3d

use common::{progress::Progress, slice::Layer};
use nalgebra::Vector2;

use crate::{mesh::Mesh, post_process::mesh_convert::marching_cubes::marching_cubes};

mod marching_cubes;
mod table;

pub fn mesh_convert(progress: &Progress, size: Vector2<u32>, result: &[Layer]) -> Mesh {
    let (vertices, faces) = marching_cubes(progress, 0.5, size, result);
    Mesh::new(vertices, faces)
}
