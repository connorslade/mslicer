pub mod bits;
pub mod png;

#[derive(Debug, Clone, Copy)]
pub struct Run<T = u8> {
    pub length: u64,
    pub value: T,
}

pub fn decode_into<T: Clone>(decoder: impl Iterator<Item = Run<T>>, image: &mut [T]) {
    let mut pixel = 0;
    for run in decoder {
        let length = run.length as usize;
        image[pixel..(pixel + length)].fill(run.value);
        pixel += length;
    }
}
