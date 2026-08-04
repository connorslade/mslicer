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
