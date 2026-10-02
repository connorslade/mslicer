// why use marching cubes when the RLE already gives us adjacency information!

use std::collections::HashMap;

use common::{
    container::rle::bits::{self, BitRunQueue},
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use nalgebra::{Vector2, Vector3};

pub struct VoxelReconstruction {
    verts: CanonicalVerts,
    faces: Vec<[u32; 3]>,

    voxel_size: Vector3<f32>,
    platform: Vector2<u32>,
}

struct CanonicalVerts {
    // maps vert to an existing index
    map: HashMap<Vector3<u32>, u32>,
    vertices: Vec<Vector3<u32>>, // todo: only store transformed float version?
    next_id: u32,
}

impl VoxelReconstruction {
    fn new(config: &SliceConfig) -> Self {
        let px = config.pixel_size().map(|x| x.get::<Milimeter>());
        let slice_height = config.slice_height.get::<Milimeter>();
        let voxel_size = Vector3::new(px.x, px.y, slice_height);

        Self {
            verts: CanonicalVerts::new(),
            faces: Vec::new(),

            voxel_size,
            platform: config.platform_resolution,
        }
    }

    pub fn into_inner(self) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
        let world = (self.verts.into_inner().into_iter())
            .map(|x| x.cast::<f32>().component_mul(&self.voxel_size))
            .collect::<Vec<_>>();

        (world, self.faces)
    }

    pub fn process(&mut self, progress: &Progress, layers: &[Layer]) {
        // layers → rows → runs
        let width = self.platform.x as u64;
        let masks = (layers.iter())
            .map(|layer| bits::chunks(&bits::from_runs(&layer.data), width))
            .collect::<Vec<_>>();

        progress.set_total(masks.len() as u64);
        for (z, rows) in masks.iter().enumerate() {
            let z = z as u32;
            let row_or_empty = |y: usize| -> &[u64] { if y < rows.len() { &rows[y] } else { &[] } };
            let mask_or_empty = |z: usize, y: usize| -> &[u64] {
                if z < masks.len() && y < rows.len() {
                    &masks[z][y]
                } else {
                    &[]
                }
            };

            for (y, row) in rows.iter().enumerate() {
                let mut x = 0;
                let y = y as u32;

                let mut prev =
                    BitRunQueue::new_fallback(row_or_empty(y.wrapping_sub(1) as usize), width);
                let mut above =
                    BitRunQueue::new_fallback(mask_or_empty(z as usize + 1, y as usize), width);
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

                    if dir ^ prev_dir {
                        self.y_face(x as u32, y, z, n as u32, dir);
                    }

                    if z == 0 && dir {
                        self.z_face(x as u32, y, 0, n as u32, false);
                    } else if dir ^ above_dir {
                        self.z_face(x as u32, y, z + 1, n as u32, dir);
                    }

                    x += n;
                }

                last.then(|| self.x_face(x as u32, y, z, true));
            }

            progress.add_complete(1);
        }
    }
}

impl VoxelReconstruction {
    fn x_face(&mut self, x: u32, y: u32, z: u32, flip: bool) {
        let a = self.verts.vertex(Vector3::new(x, y, z));
        let b = self.verts.vertex(Vector3::new(x, y + 1, z));
        let c = self.verts.vertex(Vector3::new(x, y + 1, z + 1));
        let d = self.verts.vertex(Vector3::new(x, y, z + 1));
        self.face([a, b, c, d], flip);
    }

    fn y_face(&mut self, x: u32, y: u32, z: u32, width: u32, flip: bool) {
        let a = self.verts.vertex(Vector3::new(x, y, z));
        let b = self.verts.vertex(Vector3::new(x + width, y, z));
        let c = self.verts.vertex(Vector3::new(x + width, y, z + 1));
        let d = self.verts.vertex(Vector3::new(x, y, z + 1));
        self.face([a, b, c, d], flip);
    }

    fn z_face(&mut self, x: u32, y: u32, z: u32, width: u32, flip: bool) {
        let a = self.verts.vertex(Vector3::new(x, y, z));
        let b = self.verts.vertex(Vector3::new(x + width, y, z));
        let c = self.verts.vertex(Vector3::new(x + width, y + 1, z));
        let d = self.verts.vertex(Vector3::new(x, y + 1, z));
        self.face([a, b, c, d], flip);
    }

    fn face(&mut self, [a, b, c, d]: [u32; 4], flip: bool) {
        if flip {
            self.faces.extend_from_slice(&[[a, b, c], [c, d, a]]);
        } else {
            self.faces.extend_from_slice(&[[a, c, b], [c, a, d]]);
        }
    }
}

impl CanonicalVerts {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            vertices: Vec::new(),
            next_id: 0,
        }
    }

    // Returns the vertex idx for some point, reusing an existing one if
    // possible
    pub fn vertex(&mut self, position: Vector3<u32>) -> u32 {
        if let Some(&idx) = self.map.get(&position) {
            return idx;
        }

        let idx = self.vertices.len() as u32;
        self.vertices.push(position);
        self.map.insert(position, self.next_id);
        self.next_id += 1;
        idx
    }

    pub fn into_inner(self) -> Vec<Vector3<u32>> {
        self.vertices
    }
}

pub fn reconstruct_mesh(
    progress: &Progress,
    config: &SliceConfig,
    layers: &[Layer],
) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
    let mut reconstruct = VoxelReconstruction::new(config);
    reconstruct.process(progress, layers);
    progress.set_finished();
    reconstruct.into_inner()
}
