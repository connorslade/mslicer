// why use marching cubes when the RLE already gives us adjacency information!

use common::{
    container::{
        Run,
        rle::{
            bits::{self, BitRunQueue},
            downsample::{chunks, downsample, downsample_adjacent, pad_2d},
        },
    },
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use nalgebra::{Vector2, Vector3};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::reconstruct_mesh::voxel::mesh::CanonicalVerts;

mod mesh;

pub struct VoxelReconstruction {
    verts: CanonicalVerts,
    faces: Vec<[u32; 3]>,

    voxel_size: Vector3<f32>,
    platform: Vector2<u32>,
    subsample: u8,
}

impl VoxelReconstruction {
    fn new(config: &SliceConfig, subsample: u8) -> Self {
        let px = config.pixel_size().map(|x| x.get::<Milimeter>());
        let slice_height = config.slice_height.get::<Milimeter>();
        let voxel_size = Vector3::new(px.x, px.y, slice_height)
            .component_mul(&Vector3::repeat(subsample as f32));

        Self {
            verts: CanonicalVerts::new(),
            faces: Vec::new(),

            voxel_size,
            platform: config.platform_resolution,
            subsample,
        }
    }

    pub fn into_inner(self) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
        let world = (self.verts.into_inner().into_iter())
            .map(|x| x.cast::<f32>().component_mul(&self.voxel_size))
            .collect::<Vec<_>>();

        (world, self.faces)
    }

    pub fn process(&mut self, progress: &Progress, layers: &[Layer]) {
        let width = self.platform.x.div_ceil(self.subsample as u32) as u64;

        // layers → rows → runs
        let masks = if self.subsample <= 1 {
            (layers.par_iter())
                .map(|layer| bits::chunks(&bits::from_runs(&layer.data), width))
                .collect::<Vec<_>>()
        } else {
            let voxels = self.platform.x as u64 * self.platform.y as u64;
            (layers.par_iter())
                .map(|layer| subsample(self.platform, self.subsample, layer))
                .chunks(self.subsample as usize)
                .map(|layers| {
                    let mut out = Vec::new();
                    downsample(layers.iter(), voxels, &mut out);
                    out
                })
                .map(|data| bits::chunks(&bits::from_runs(&data), width))
                .collect::<Vec<_>>()
        };

        progress.set_total(masks.len() as u64);
        for (z, rows) in masks.iter().enumerate() {
            let z = z as u32;
            let get_row = |y: usize| -> &[u64] { if y < rows.len() { &rows[y] } else { &[] } };
            let get_mask = |z: usize, y: usize| -> &[u64] {
                if z < masks.len() && y < rows.len() {
                    &masks[z][y]
                } else {
                    &[]
                }
            };

            for (y, row) in rows.iter().enumerate() {
                let mut x = 0;
                let y = y as u32;

                let (prev_y, next_z) = ((y as usize).wrapping_sub(1), z as usize + 1);
                let mut prev = BitRunQueue::new_fallback(get_row(prev_y), width);
                let mut above = BitRunQueue::new_fallback(get_mask(next_z, y as usize), width);
                let mut row = BitRunQueue::new(row);

                let mut last = false;
                while row.remaining() {
                    let n = (row.active.length)
                        .min(prev.active.length)
                        .min(above.active.length);
                    let prev_dir = prev.take_up_to(n).value;
                    let above_dir = above.take_up_to(n).value;
                    let dir = row.take_up_to(n).value;

                    if dir ^ last {
                        last = dir;
                        self.x_face(x as u32, y, z, !dir);
                    }

                    (dir ^ prev_dir).then(|| self.y_face(x as u32, y, z, n as u32, dir));
                    (z == 0 && dir).then(|| self.z_face(x as u32, y, 0, n as u32, false));
                    (dir ^ above_dir).then(|| self.z_face(x as u32, y, z + 1, n as u32, dir));

                    x += n;
                }

                last.then(|| self.x_face(x as u32, y, z, true));
            }

            progress.add_complete(1);
        }
    }
}

fn subsample(res: Vector2<u32>, factor: u8, layer: &Layer) -> Vec<Run> {
    let (data, size) = pad_2d(&layer.data, res, factor, 0);

    let mut out = Vec::new();
    downsample_adjacent(factor, &data, &mut out);
    let chunks = chunks(&out, size.x as u64 / factor as u64);

    let mut out = Vec::new();
    for y in chunks.chunks(factor as usize) {
        downsample(y.iter(), size.x as u64 / factor as u64, &mut out);
    }

    out
}

pub fn reconstruct_mesh(
    progress: &Progress,
    config: &SliceConfig,
    layers: &[Layer],
    subsample: u8,
) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
    let mut reconstruct = VoxelReconstruction::new(config, subsample);
    reconstruct.process(progress, layers);
    progress.set_finished();
    reconstruct.into_inner()
}
