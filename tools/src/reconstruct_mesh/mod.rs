//! Marching cubes implementation is modified from my wave-sim-3d project.
//! https://github.com/connorslade/wave-sim-3d

use std::time::Instant;

use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};

mod marching_cubes;
mod table;

use slicer::mesh::Mesh;
use tracing::info;

pub fn marching_cubes(progress: &Progress, config: &SliceConfig, result: &[Layer]) -> Mesh {
    let start = Instant::now();
    let (vertices, faces) = marching_cubes::marching_cubes(progress, 0.5, config, result, 5);
    info!("Reconstructed mesh in {:?}", start.elapsed());
    Mesh::new(vertices, faces)
}
