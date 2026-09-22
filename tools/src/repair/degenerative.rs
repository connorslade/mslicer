use crate::repair::{MeshRepair, RepairState};

impl MeshRepair {
    pub(super) fn repair_degenerative_faces(&self, state: &mut RepairState) -> u64 {
        // Check if any face has zero area (any two vertices are the same)

        let mut count = 0;
        state.faces.retain(|[a, b, c]| {
            let degenerative = a == b || b == c || a == c;
            count += degenerative as u64;
            !degenerative
        });

        count
    }
}
