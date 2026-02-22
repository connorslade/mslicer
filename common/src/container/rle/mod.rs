//! Tools for working with run length encoded (RLE) data.

pub mod bits;
pub mod png;

/// Sequence of identical items.
#[derive(Debug, Clone, Copy)]
pub struct Run<T = u8> {
    pub length: u64,
    pub value: T,
}

/// Decode a RLE sequence into a mutable slice.
pub fn decode_into<T: Clone>(decoder: impl Iterator<Item = Run<T>>, image: &mut [T]) {
    let mut pixel = 0;
    for run in decoder {
        let length = run.length as usize;
        image[pixel..(pixel + length)].fill(run.value);
        pixel += length;
    }
}
