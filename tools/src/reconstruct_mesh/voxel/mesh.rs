use std::collections::HashMap;

use nalgebra::Vector3;

use crate::reconstruct_mesh::voxel::VoxelReconstruction;

pub struct CanonicalVerts {
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

impl VoxelReconstruction {
    pub(super) fn x_face(&mut self, x: u32, y: u32, z: u32, flip: bool) {
        let a = self.verts.vertex(Vector3::new(x, y, z));
        let b = self.verts.vertex(Vector3::new(x, y + 1, z));
        let c = self.verts.vertex(Vector3::new(x, y + 1, z + 1));
        let d = self.verts.vertex(Vector3::new(x, y, z + 1));
        self.face([a, b, c, d], flip);
    }

    pub(super) fn y_face(&mut self, x: u32, y: u32, z: u32, width: u32, flip: bool) {
        let a = self.verts.vertex(Vector3::new(x, y, z));
        let b = self.verts.vertex(Vector3::new(x + width, y, z));
        let c = self.verts.vertex(Vector3::new(x + width, y, z + 1));
        let d = self.verts.vertex(Vector3::new(x, y, z + 1));
        self.face([a, b, c, d], flip);
    }

    pub(super) fn z_face(&mut self, x: u32, y: u32, z: u32, width: u32, flip: bool) {
        let a = self.verts.vertex(Vector3::new(x, y, z));
        let b = self.verts.vertex(Vector3::new(x + width, y, z));
        let c = self.verts.vertex(Vector3::new(x + width, y + 1, z));
        let d = self.verts.vertex(Vector3::new(x, y + 1, z));
        self.face([a, b, c, d], flip);
    }

    pub(super) fn face(&mut self, [a, b, c, d]: [u32; 4], flip: bool) {
        if flip {
            self.faces.extend_from_slice(&[[a, b, c], [c, d, a]]);
        } else {
            self.faces.extend_from_slice(&[[a, c, b], [c, a, d]]);
        }
    }
}
