use std::collections::HashMap;

use common::{
    container::rle::bits,
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use itertools::Itertools;
use nalgebra::Vector3;

pub fn reconstruct_mesh(
    progress: &Progress,
    config: &SliceConfig,
    result: &[Layer],
    subsample: u8, // todo: bit-rle downsampling???
) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
    let mut verts = CanonicalVerts::new();
    let mut faces = Vec::new();

    let mut row_face = |x, y, z, flip| {
        let a = verts.vertex(Vector3::new(x, y, z));
        let b = verts.vertex(Vector3::new(x, y + 1, z));
        let c = verts.vertex(Vector3::new(x, y + 1, z + 1));
        let d = verts.vertex(Vector3::new(x, y, z + 1));
        if flip {
            faces.extend_from_slice(&[[a, b, c], [c, d, a]]);
        } else {
            faces.extend_from_slice(&[[a, c, b], [c, a, d]]);
        }
    };

    progress.set_total(result.len() as u64);
    for (z, layer) in result.iter().enumerate() {
        let z = z as u32;
        let mask = bits::from_runs(&layer.data);
        let rows = bits::chunks(&mask, config.platform_resolution.x as u64);

        for (y, row) in rows.into_iter().enumerate() {
            let mut x = 0;
            let y = y as u32;

            for (black, white) in row.iter().tuples() {
                x += black;
                row_face(x as u32, y, z, false);
                x += white;
                row_face(x as u32, y, z, true);
            }
        }

        progress.add_complete(1);
    }

    // for screen space → world space
    let px = config.pixel_size().map(|x| x.get::<Milimeter>());
    let slice_height = config.slice_height.get::<Milimeter>();
    let voxel_size = Vector3::new(px.x, px.y, slice_height);

    let world = verts
        .into_inner()
        .into_iter()
        .map(|x| x.cast::<f32>().component_mul(&voxel_size))
        .collect::<Vec<_>>();

    progress.set_finished();

    println!("{{ face: {}, vert: {} }}", faces.len(), world.len());

    (world, faces)
}

struct CanonicalVerts {
    // maps vert to an existing index
    map: HashMap<Vector3<u32>, u32>,
    vertices: Vec<Vector3<u32>>, // todo: only store transformed float version?
    next_id: u32,
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
