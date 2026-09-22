use std::collections::HashMap;

use nalgebra::Vector3;

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

        // Full the hole by forming faces between the existing vertices on the hole.
        for edge_loop in loops.iter() {
            min_area_triangulation(&state.vertices, edge_loop, &mut state.faces);
        }

        loops.len() as u64
    }
}

// find a triangulation that gives the smallest sum triangle area
fn min_area_triangulation(points: &[Vector3<f32>], hole: &[u32], faces: &mut Vec<[u32; 3]>) {
    let len = hole.len();
    let mut dp = vec![vec![0.0; len]; len];
    let mut parent = vec![vec![None; len]; len];

    // calculate total cost of sub polygons
    for n in 2..len {
        for i in 0..len - n {
            let j = i + n;
            let [vi, vj] = [i, j].map(|i| points[hole[i] as usize]);

            dp[i][j] = f32::INFINITY;
            for k in i + 1..j {
                let vk = points[hole[k] as usize];
                let cost = (vi - vj).cross(&(vk - vj)).magnitude();
                let total_cost = cost + dp[i][k] + dp[k][j];

                if total_cost < dp[i][j] {
                    dp[i][j] = total_cost;
                    parent[i][j] = Some(k);
                }
            }
        }
    }

    // reconstruct best solution from dp table
    let mut stack = vec![[0, len - 1]];
    while let Some([i, j]) = stack.pop() {
        if j > i + 1
            && let Some(k) = parent[i][j]
        {
            faces.push([j, k, i].map(|x| hole[x]));
            stack.push([i, k]);
            stack.push([k, j]);
        }
    }
}
