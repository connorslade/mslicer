//! Tools for working with run length encoded (RLE) data.

use std::borrow::Borrow;

pub mod bits;
pub mod downsample;
pub mod png;

/// Sequence of identical items.
#[derive(Debug, Clone, Copy)]
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
