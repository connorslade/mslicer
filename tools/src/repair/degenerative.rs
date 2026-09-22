use std::collections::HashSet;

use crate::repair::{MeshRepair, RepairState};

impl MeshRepair {
    // check if any face has zero area (any two vertices are the same) or
    // the face was defined previously.
    pub(super) fn repair_degenerative_faces(&self, state: &mut RepairState) -> (u64, u64) {
        let mut seen = HashSet::new();

        let mut degenerative_count = 0;
        let mut repeated_count = 0;
        state.faces.retain(|f @ [a, b, c]| {
            if a == b || b == c || a == c {
                degenerative_count += 1;
                return false;
            }

            if !seen.insert(key(*f)) {
                repeated_count += 1;
                return false;
            }

            true
        });

        (repeated_count, degenerative_count)
    }
}

fn key(mut face: [u32; 3]) -> [u32; 3] {
    face.sort();
    face
}
