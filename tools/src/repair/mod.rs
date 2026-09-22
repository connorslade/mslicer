use std::fmt::Debug;

use nalgebra::Vector3;
use slicer::{half_edge::HalfEdgeMesh, mesh::Mesh};

mod degenerative;
mod holes;
mod unwelded_vertices;
mod winding_order;

// todo: separate and faster detection pass?

pub struct MeshRepair {
    pub vertex_epsilon: f32,
}

pub struct RepairResult {
    pub mesh: Mesh,

    pub unwelded_vertices: u64,
    pub degenerative_faces: u64,
    pub repeated_faces: u64,
    pub holes: u64,
    pub flipped_winding: u64,
}

struct RepairState {
    vertices: Vec<Vector3<f32>>,
    faces: Vec<[u32; 3]>,
    // todo: store working_* vecs here to maintain the same allocation for processing
}

impl MeshRepair {
    pub fn repair(&self, mesh: &Mesh) -> RepairResult {
        // mesh must be cloned since it will be modified
        let mut state = RepairState {
            vertices: mesh.vertices().to_vec(),
            faces: mesh.faces().to_vec(),
        };

        let unwelded_vertices = self.repair_unwelded_vertices(&mut state);
        let (repeated_faces, degenerative_faces) = self.repair_degenerative_faces(&mut state);
        let flipped_winding = self.repair_winding_order(&mut state);

        let half_edge = HalfEdgeMesh::build(state.faces.iter());
        let holes = self.repair_holes(&mut state, &half_edge);

        let mut out = Mesh::new(state.vertices, state.faces);
        out.set_position(mesh.position());
        out.set_rotation(mesh.rotation());
        out.set_scale(mesh.scale());
        RepairResult {
            mesh: out,

            unwelded_vertices,
            degenerative_faces,
            repeated_faces,
            holes,
            flipped_winding,
        }
    }
}

impl RepairResult {
    pub fn any_defects(&self) -> bool {
        self.degenerative_faces > 0
            || self.holes > 0
            || self.degenerative_faces > 0
            || self.flipped_winding > 0
    }
}

impl Debug for RepairResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RepairResult")
            .field("unwelded_vertices", &self.unwelded_vertices)
            .field("degenerative_faces", &self.degenerative_faces)
            .field("repeated_faces", &self.repeated_faces)
            .field("holes", &self.holes)
            .field("flipped_winding", &self.flipped_winding)
            .finish()
    }
}
