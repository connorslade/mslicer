use std::sync::Arc;

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
    pub holes: u64,
    pub degenerative_faces: u64,
    pub flipped_winding: u64,
}

struct RepairState {
    vertices: Vec<Vector3<f32>>,
    faces: Vec<[u32; 3]>,
    half_edge: Arc<HalfEdgeMesh>,
    // todo: store working_* vecs here to maintain the same allocation for processing
}

impl MeshRepair {
    pub fn repair(&self, mesh: &Mesh, half_edge: Arc<HalfEdgeMesh>) -> RepairResult {
        // mesh must be cloned since it will be modified
        let mut state = RepairState {
            vertices: mesh.vertices().to_vec(),
            faces: mesh.faces().to_vec(),
            half_edge,
        };

        let holes = self.repair_holes(&mut state); // todo: should run after welding
        let unwelded_vertices = self.repair_unwelded_vertices(&mut state);
        let degenerative_faces = self.repair_degenerative_faces(&mut state);
        let flipped_winding = self.repair_winding_order(&mut state);

        let mut out = Mesh::new(state.vertices, state.faces);
        out.set_position(mesh.position());
        out.set_rotation(mesh.rotation());
        out.set_scale(mesh.scale());
        RepairResult {
            mesh: out,
            unwelded_vertices,
            holes,
            degenerative_faces,
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
