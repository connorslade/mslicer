use std::collections::{HashMap, HashSet};

use crate::mesh::Mesh;

pub struct HalfEdgeMesh<'a> {
    mesh: &'a Mesh,
    half_edges: Vec<HalfEdge>,
}

#[derive(Debug)]
pub struct HalfEdge {
    origin_vertex: u32,
    vertex: u32,
    face: u32,

    next: u32,
    prev: u32,
    twin: Option<u32>,
}

impl<'a> HalfEdgeMesh<'a> {
    pub fn new(mesh: &'a Mesh) -> Self {
        let mut half_edges = Vec::new();
        let mut edge_map = HashMap::new();

        for (face_idx, face) in mesh.faces().iter().enumerate() {
            let first_edge = half_edges.len() as u32;
            for i in 0..3 {
                let next = first_edge + (i as u32 + 1) % 3;
                let prev = first_edge + (i as u32 + 2) % 3;

                let half_edge = HalfEdge {
                    origin_vertex: face[i],
                    vertex: face[(i + 1) % 3],
                    face: face_idx as u32,

                    next,
                    prev,
                    twin: None,
                };

                let edge_key = (face[i], face[(i + 1) % 3]);
                half_edges.push(half_edge);
                edge_map.insert(edge_key, first_edge + i as u32);
            }
        }

        for edge in half_edges.iter_mut() {
            edge.twin = edge_map.get(&(edge.vertex, edge.origin_vertex)).copied();
        }

        Self { mesh, half_edges }
    }

    pub fn half_edges(&self) -> &[HalfEdge] {
        &self.half_edges
    }

    /// Returns a set of all edges connected to the given vertex.
    pub fn connected_vertices(&self, start_edge: u32) -> Vec<u32> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        let mut edge = start_edge;

        loop {
            if !seen.insert(edge) {
                return out;
            }
            out.push(self.half_edges[edge as usize].vertex);

            let Some(this_edge) = self.half_edges[edge as usize].twin else {
                break;
            };
            edge = self.half_edges[this_edge as usize].next;
            if this_edge == start_edge {
                break;
            }
        }

        out
    }

    pub fn vertex(&self, idx: u32) -> u32 {
        self.half_edges[idx as usize].vertex
    }
}
