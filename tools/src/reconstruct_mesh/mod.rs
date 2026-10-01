//! Marching cubes implementation is modified from my wave-sim-3d project.
//! https://github.com/connorslade/wave-sim-3d

use std::time::Instant;

use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};

mod marching_cubes;
mod voxel;

use slicer::mesh::Mesh;
use tracing::info;

pub fn marching_cubes(
    progress: &Progress,
    config: &SliceConfig,
    result: &[Layer],
    subsample: u8,
) -> Mesh {
    let start = Instant::now();
    let (vertices, faces) =
        marching_cubes::reconstruct_mesh(progress, 0.5, config, result, subsample);
    info!(
        "Reconstructed mesh (marching_cubes) in {:?}",
        start.elapsed()
    );
    Mesh::new(vertices, faces)
}

pub fn greedy_rle(progress: &Progress, config: &SliceConfig, result: &[Layer]) -> Mesh {
    let start = Instant::now();
    let (vertices, faces) = voxel::reconstruct_mesh(progress, config, result);
    info!("Reconstructed mesh (greedy_rle) in {:?}", start.elapsed());
    Mesh::new(vertices, faces)
}
