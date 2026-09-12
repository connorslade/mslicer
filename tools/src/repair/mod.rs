use std::{collections::HashMap, iter, sync::Arc};

use itertools::Itertools;
use nalgebra::Vector3;
use slicer::{half_edge::HalfEdgeMesh, mesh::Mesh};

// todo: separate and faster detection pass?

pub struct MeshRepair {
    pub vertex_epsilon: f32,
}

pub struct RepairResult {
    pub mesh: Mesh,

    pub unwelded_vertices: u64,
    pub holes: u64,
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

        let holes = self.repair_holes(&mut state);
        let unwelded_vertices = self.repair_unwelded_vertices(&mut state);

        let mut out = Mesh::new(state.vertices, state.faces);
        out.set_position(mesh.position());
        out.set_rotation(mesh.rotation());
        out.set_scale(mesh.scale());
        RepairResult {
            mesh: out,
            unwelded_vertices,
            holes,
        }
    }

    fn repair_holes(&self, state: &mut RepairState) -> u64 {
        let mut map = HashMap::new();
        for edge in state.half_edge.half_edges() {
            // if edge only has one face, edge is on a hole
            if edge.twin.is_none() {
                map.insert(edge.origin_vertex, edge.vertex);
            }
        }

        let mut loops = Vec::new();
        while !map.is_empty() {
            let start = *map.keys().next().unwrap();
            let mut edge_loop = Vec::new();
            let mut pointer = start;

            loop {
                edge_loop.push(pointer);
                let Some(next) = map.remove(&pointer) else {
                    break;
                };

                if next == start {
                    break;
                }

                pointer = next
            }

            loops.push(edge_loop);
        }

        for edge_loop in loops.iter() {
            let center = (edge_loop.iter())
                .map(|x| state.vertices[*x as usize])
                .sum::<Vector3<_>>()
                / edge_loop.len() as f32;

            let c = state.vertices.len() as u32;
            state.vertices.push(center);

            let first = &edge_loop[0];
            for (a, b) in edge_loop.iter().chain(iter::once(first)).tuple_windows() {
                state.faces.push([*a, c, *b]);
            }
        }

        loops.len() as u64
    }

    fn repair_unwelded_vertices(&self, state: &mut RepairState) -> u64 {
        // Detect close vertices
        let mut spatial = HashMap::<_, Vec<_>>::new();
        for (i, vert) in state.vertices.iter().enumerate() {
            // floor needed since it rounds negative numbers away (↓) from zero
            let hash = vert.map(|x| (x / self.vertex_epsilon).floor() as i64);
            spatial.entry(hash).or_default().push(i as u32);
        }

        // Reconstruct mesh from spatial hash
        let mut tree = HashMap::new(); // todo: use cluster data structure
        let epsilon_sq = self.vertex_epsilon.powi(2);
        for (cell, vertices) in spatial.iter() {
            // todo: all vertices in a single cell are already known to be repeated

            for ((dx, dy), dz) in (-1..=1).cartesian_product(-1..=1).cartesian_product(-1..=1) {
                let hash = Vector3::new(cell.x + dx, cell.y + dy, cell.z + dz);
                if let Some(other_vertices) = spatial.get(&hash) {
                    for a in vertices.iter() {
                        for b in other_vertices.iter() {
                            // this is the most nested loop ive ever written...
                            // this is going to be so slow... we can optimize
                            // later ig...

                            if a < b
                                && (state.vertices[*a as usize] - state.vertices[*b as usize])
                                    .magnitude_squared()
                                    < epsilon_sq
                            {
                                tree.insert(*a.max(b), *a.min(b));
                            }
                        }
                    }
                }
            }
        }

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        let mut rename = HashMap::new();
        for i in 0..state.vertices.len() as u32 {
            let mut target = i;
            while let Some(&next) = tree.get(&target) {
                target = next;
            }

            let new_index = *rename.entry(target).or_insert_with(|| {
                let idx = vertices.len() as u32;
                vertices.push(state.vertices[target as usize]);
                idx
            });
            rename.insert(i, new_index);
        }

        for face in state.faces.iter() {
            faces.push(face.map(|x| {
                rename
                    .get(&tree.get(&x).copied().unwrap_or(x))
                    .copied()
                    .unwrap()
            }));
        }

        let unwelded = state.vertices.len() - vertices.len();
        state.vertices = vertices;
        state.faces = faces;

        unwelded as u64
    }
}
