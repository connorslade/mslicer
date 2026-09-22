use std::{
    collections::{HashMap, VecDeque},
    iter, mem,
};

use itertools::Itertools;

use crate::repair::{MeshRepair, RepairState};

impl MeshRepair {
    // First make sure there is a constant winding order, then flip global to give a positive signed area
    pub(super) fn repair_winding_order(&self, state: &mut RepairState) -> u64 {
        // find what triangles each edge touches. In the case of two triangles,
        // each should have opposing edge winding. Not sure about when n > 2...
        let mut edges = HashMap::<_, Vec<_>>::new();
        for (i, face) in state.faces.iter().enumerate() {
            for (a, b) in face_edges(face) {
                let key = (a.min(b), a.max(b));
                edges.entry(key).or_default().push(i);
            }
        }

        // iterate through all the faces, flipping ones that aren't consistent.
        let n = state.faces.len();
        let mut count = 0;
        let mut seen = vec![false; n];

        for start in 0..n {
            // ignore visited faces
            if mem::replace(&mut seen[start], true) {
                continue;
            }

            let mut queue = VecDeque::from([start]);
            while let Some(next) = queue.pop_front() {
                for edge @ (a, b) in face_edges(&state.faces[next]) {
                    let key = (a.min(b), a.max(b));
                    for face in &edges[&key] {
                        if mem::replace(&mut seen[*face], true) {
                            continue;
                        }

                        // flip winding order if it is inconsistent with next
                        if matching_winding(&state.faces[*face], edge) {
                            state.faces[*face].swap(1, 2);
                            count += 1;
                        }

                        queue.push_back(*face);
                    }
                }
            }
        }

        // todo: flip all if has an area < 0

        count
    }
}

fn face_edges(face: &[u32; 3]) -> Vec<(u32, u32)> {
    face.iter()
        .chain(iter::once(&face[0]))
        .copied()
        .tuple_windows()
        .collect()
}

fn matching_winding([a, b, c]: &[u32; 3], (alpha, beta): (u32, u32)) -> bool {
    (*a == alpha && *b == beta) || (*b == alpha && *c == beta) || (*c == alpha && *a == beta)
}
