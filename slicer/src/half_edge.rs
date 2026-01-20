use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::mesh::MeshInner;

#[derive(Clone)]
pub struct HalfEdgeMesh {
    half_edges: Vec<HalfEdge>,
}

#[derive(Debug, Clone)]
pub struct HalfEdge {
    pub origin_vertex: u32,
    pub vertex: u32,
    pub face: u32,

    pub next: u32,
    pub prev: u32,
    pub twin: Option<u32>,
}

impl HalfEdgeMesh {
    pub fn build(mesh: &Arc<MeshInner>) -> Self {
        let mut half_edges = Vec::new();
        let mut edge_map = HashMap::new();

        for (face_idx, face) in mesh.faces.iter().enumerate() {
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

        Self { half_edges }
    }

    pub fn half_edges(&self) -> &[HalfEdge] {
        &self.half_edges
    }

    pub fn half_edge_count(&self) -> usize {
        self.half_edges.len()
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
            if edge == start_edge {
                break;
            }
        }

        out
    }

    pub fn vertex(&self, idx: u32) -> u32 {
        self.half_edges[idx as usize].origin_vertex
    }

    pub fn get_edge(&self, idx: u32) -> &HalfEdge {
        &self.half_edges[idx as usize]
    }
}
