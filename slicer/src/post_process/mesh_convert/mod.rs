//! Marching cubes implementation is modified from my wave-sim-3d project.
//! https://github.com/connorslade/wave-sim-3d

// todo: move to tools

use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};

use crate::{mesh::Mesh, post_process::mesh_convert::marching_cubes::marching_cubes};

mod marching_cubes;
mod table;

pub fn mesh_convert(progress: &Progress, config: &SliceConfig, result: &[Layer]) -> Mesh {
    let (vertices, faces) = marching_cubes(progress, 0.5, config, result, 5);
    Mesh::new(vertices, faces)
}
