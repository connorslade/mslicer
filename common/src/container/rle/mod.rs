//! Tools for working with run length encoded (RLE) data.

use std::{borrow::Borrow, iter::repeat_n};

use crate::container::rle::downsample::RunQueue;

pub mod bits;
pub mod downsample;
pub mod png;

/// Sequence of identical items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run<T = u8> {
    pub length: u64,
    pub value: T,
}

impl<T> Run<T> {
    pub fn new(length: u64, value: T) -> Self {
        Self { length, value }
    }
}

/// Decode a RLE sequence into a mutable slice.
pub fn decode_into<T, R, D>(decoder: D, image: &mut [T])
where
    T: Clone,
    R: Borrow<Run<T>>,
    D: IntoIterator<Item = R>,
{
    let mut pixel = 0;
    for run in decoder {
        let run = run.borrow();
        let length = run.length as usize;
        image[pixel..(pixel + length)].fill(run.value.clone());
        pixel += length;
    }
}

pub fn decode_vec<T, R, D>(decoder: D) -> Vec<T>
where
    T: Clone,
    R: Borrow<Run<T>>,
    D: IntoIterator<Item = R>,
{
    let mut out = Vec::new();
    for run in decoder {
        let run = run.borrow();
        let length = run.length as usize;
        out.extend(repeat_n(run.value.clone(), length));
    }

    out
}

/// Finds the difference between two RLE sequences. That is the sum of all pixel
/// value differences (abs) between the two sequences.
///
/// Assumes the (uncompressed) input data are equal length.
pub fn difference(a: &[Run], b: &[Run]) -> u64 {
    let (mut a, mut b) = (RunQueue::new(a), RunQueue::new(b));
    let mut difference = 0;

    while a.remaining() || b.remaining() {
        let len = a.active.length.min(b.active.length);
        let (a, b) = (a.take_up_to(len), b.take_up_to(len));
        difference += a.value.abs_diff(b.value) as u64 * len;
    }

    difference
}

/// Adds a value into a RLE encoded sequence with a specified spacing.
///
/// For example running on `[0×3,1×2,0×3]` with `{ instance: 1×2, start: 0,
/// spacing: 2}`, you will get `[1×2, 0×2, 1×2, 0×1, 1×4, 0×1, 1×2, 0×2]`.
pub fn intersperse_runs(runs: &mut Vec<Run>, instance: Run, start: u64, spacing: u64) {
    let mut i = 0; // The current run being processed
    let mut pos = 0; // The current position in bytes
    let mut next = start; // Next byte index to insert `value`

    while i < runs.len() {
        let run = &mut runs[i];

        // The range of positions covered by the current run. Excluding end.
        // [start, pos)
        let (start, end) = (pos, pos + run.length);

        // If next insertion point is not in the range, advance to the next run.
        // But if it is, split the run into parts left and right of the
        // insertion point with the inserted run between.
        if (start..end).contains(&next) {
            // Avoid splitting run into parts if possible. When the values are
            // the same, the length can just be updated.
            if run.value == instance.value {
                let n = 1 + (end - next - 1) / spacing;
                pos += run.length;
                next += spacing * n;
                run.length += n * instance.length;
                i += 1;
            } else {
                let run = runs.remove(i);

                let length_left = next - start;
                let length_right = run.length - length_left;
                next += spacing;
                pos += length_left;

                if length_left > 0 {
                    let (length, value) = (length_left, run.value);
                    runs.insert(i, Run { length, value });
                    i += 1;
                }

                runs.insert(i, instance);
                i += 1;

                if length_right > 0 {
                    let (length, value) = (length_right, run.value);
                    runs.insert(i, Run { length, value });
                }
            }
        } else {
            pos += run.length;
            i += 1;
        }
    }
}
