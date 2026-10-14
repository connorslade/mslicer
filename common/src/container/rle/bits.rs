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
            // Start the row with a zero if the next value to be inserted is non zero.
            (row.is_empty() && i % 2 != 0).then(|| row.push(0));

            // Add as much of the current run to the active row as will fit.
            let clamped = run.min(width - row_length);
            row.push(clamped);
            row_length += clamped;
            run -= clamped;

            // If the current run is now full, flush it and initialize the next.
            if row_length >= width {
                rows.push(mem::take(&mut row));
                row_length = 0;
            }
        }
    }

    // Flush the final row in the case that the input data length is not a
    // multiple of `width`.
    (!row.is_empty()).then(|| rows.push(row));

    rows
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ClusterRun {
    pub row: usize,
    pub index: usize,

    pub position: u64,
    pub size: u64,
}

/// Inserts all the adjacencies found between the two rows (`curr` and `base`)
/// into the container.
pub fn cluster_row_adjacency(
    clusters: &mut Clusters<ClusterRun>,
    rows: &[Vec<u64>],
    base_row: usize,
    curr_row: usize,
) {
    let (base, curr) = (&rows[base_row], &rows[curr_row]);

    let mut b_pos = 0;
    for (i, &b) in curr.iter().enumerate() {
        if i % 2 != 0 {
            let b_end = b_pos + b;

            let cluster_b = ClusterRun {
                row: curr_row,
                index: i,
                position: b_pos,
                size: b,
            };
            clusters.insert(cluster_b);

            let mut a_pos = 0;
            for (j, &a) in base.iter().enumerate() {
                let a_end = a_pos + a;
                if a_pos > b_end {
                    break;
                }

                if j % 2 != 0 && b_pos <= a_end && b_end >= a_pos {
                    let cluster_a = ClusterRun {
                        row: base_row,
                        index: j,
                        position: a_pos,
                        size: a,
                    };
                    clusters.mark_adjacency(cluster_a, cluster_b);
                }

                a_pos += a;
            }
        }

        b_pos += b;
    }
}

pub struct BitRunQueue<'a> {
    pub runs: &'a [u64],
    pub next: usize,
    pub active: BitRun,
}

#[derive(Clone, Copy)]
pub struct BitRun {
    pub value: bool,
    pub length: u64,
}

impl<'a> BitRunQueue<'a> {
    pub fn new(runs: &'a [u64]) -> Self {
        if runs[0] == 0 && runs.len() > 1 {
            Self {
                runs,
                next: 2,
                active: BitRun::new(runs[1], true),
            }
        } else {
            Self {
                runs,
                next: 1,
                active: BitRun::new(runs[0], false),
            }
        }
    }

    pub fn new_fallback(runs: &'a [u64], width: u64) -> Self {
        if runs.is_empty() {
            Self {
                runs,
                next: 1,
                active: BitRun::new(width, false),
            }
        } else {
            Self::new(runs)
        }
    }

    pub fn advance(&mut self) -> BitRun {
        let out = self.active;
        if self.next < self.runs.len() {
            self.active = BitRun::new(self.runs[self.next], !self.next.is_multiple_of(2));
            self.next += 1;
        } else {
            self.active.length = 0;
        }
        out
    }

    pub fn take_up_to(&mut self, n: u64) -> BitRun {
        if self.active.length <= n {
            self.advance()
        } else {
            self.active.length -= n;
            BitRun::new(n, self.active.value)
        }
    }

    pub fn remaining(&self) -> bool {
        self.active.length > 0
    }
}

impl BitRun {
    pub fn new(length: u64, value: bool) -> Self {
        Self { value, length }
    }
}
