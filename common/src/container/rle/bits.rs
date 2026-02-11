//! Algorithms for working with run length encoded bit streams (BitRLE). They
//! are used when analyzing layer data in cases where exact pixel values don't
//! matter, just weather they are on of off.

use std::mem;

use crate::container::{Clusters, Run};

/// Converts a RLE byte stream into a RLE bit mask of nonzero values.
///
/// Returns a list of lengths, starting with zero and alternating. So `[0, 23,
/// 7]` would mean the run starts with 23 non-zero bytes, then 7 zero bytes.
pub fn from_runs(runs: &[Run]) -> Vec<u64> {
    let mut out = Vec::with_capacity(runs.len());

    let mut value = false;
    let mut length = 0;
    for run in runs {
        let this_value = run.value > 0;
        if this_value ^ value {
            out.push(mem::replace(&mut length, run.length));
            value = this_value;
        } else {
            length += run.length;
        }
    }

    (length > 0).then(|| out.push(length));
    out
}

/// Split a run length encoded bit stream into chunks each `width` bits long.
pub fn chunks(runs: &[u64], width: u64) -> Vec<Vec<u64>> {
    let mut rows = Vec::new();

    let mut row = Vec::new();
    let mut row_length = 0;

    for (i, mut run) in runs.iter().copied().enumerate() {
        while run > 0 {
            // Add as much of the current run to the active row as will fit.
            let clamped = run.min(width - row_length);
            row.push(clamped);
            row_length += clamped;
            run -= clamped;

            // If the current run is now full, flush it and initialize the next.
            // Start the row with a zero if the next value to be inserted is non
            // zero.
            if row_length >= width {
                rows.push(mem::take(&mut row));
                row_length = 0;
                ((i % 2 == 0) ^ (run > 0)).then(|| row.push(0));
            }
        }
    }

    // Flush the final row in the case that the input data length is not a
    // multiple of `width`.
    (!row.is_empty()).then(|| rows.push(row));

    rows
}

/// Inserts all the adjacencies found between the two rows (`curr` and `base`)
/// into the container. The run type is a tuple: (row, index, size).
pub fn cluster_row_adjacency(
    cluster: &mut Clusters<(usize, usize, u64)>,
    rows: &[Vec<u64>],
    base_row: usize,
    curr_row: usize,
) {
    let (base, curr) = (&rows[base_row], &rows[curr_row]);

    let mut b_pos = 0;
    for (i, &b) in curr.iter().enumerate() {
        if i % 2 != 0 {
            let b_end = b_pos + b;

            let mut a_pos = 0;
            for (j, &a) in base.iter().enumerate() {
                let a_end = a_pos + a;
                if a_pos > b_end {
                    break;
                }

                if j % 2 != 0 && b_pos <= a_end && b_end >= a_pos {
                    cluster.mark_adjacency((base_row, j, a), (curr_row, i, b));
                }

                a_pos += a;
            }
        }

        b_pos += b;
    }
}
