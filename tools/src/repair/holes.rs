use std::collections::HashMap;

use ordered_float::OrderedFloat;

use crate::repair::{MeshRepair, RepairState};

impl MeshRepair {
    pub(super) fn repair_holes(&self, state: &mut RepairState) -> u64 {
        let mut map = HashMap::new();
        for edge in state.half_edge.half_edges() {
            // if edge only has one face, edge is on a hole. Add it as a
            // directed edge to the loop map.
            if edge.twin.is_none() {
                map.insert(edge.origin_vertex, edge.vertex);
            }
        }

        // Flatten map into lists of hole edge loops
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

        // Full the hole by forming faces between the existing vertices on the
        // hole. Optimized for minimum face area, shouldn't cause too many
        // issues with self-intersections.
        for edge_loop in loops.iter() {
            let mut poly = edge_loop.clone();
            while poly.len() > 3 {
                let n = poly.len();
                let i = (0..n)
                    .min_by_key(|&idx| {
                        let face = state.faces[idx];
                        let [a, b, c] = face.map(|x| state.vertices[x as usize]);
                        OrderedFloat((a - b).cross(&(c - b)).magnitude()) // minimizes triangle area
                    })
                    .unwrap();

                let face = [i, (i + n - 1) % n, (i + 1) % n].map(|k| poly[k]);
                state.faces.push(face);
                poly.remove(i);
            }
            state.faces.push([poly[1], poly[2], poly[0]]);
        }

        loops.len() as u64
    }
}
