//! Custom PNG encoder optimized for encoding RLE byte streams. This was
//! probably a lot more work than it was worth. It was interesting to write at
//! least :eyes:.
//!
//! ## References
//!
//! - PNG
//!   - https://www.bamfordresearch.com/2021/one-hour-png
//!   - https://www.libpng.org/pub/png/book/chapter11.html
//! - Deflate
//!   - https://www.rfc-editor.org/rfc/rfc1951
//!   - https://github.com/image-rs/fdeflate/blob/c365c7e6ffa81feb2e1fb762eed7299f05c9b0ca/src/compress.rs
//!   - https://gist.github.com/juliusgeo/d4965b16a3c4478bb4eca2fe210559eb

use common::{
    bitvec::BitVec,
    misc::Run,
    serde::{DynamicSerializer, Serializer},
};

use deflate::{Adler32, huffman, lz77_compress};
pub mod deflate;

const MAGIC: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];

pub struct PngEncoder<'a> {
    header: &'a PngInfo,
    planes: u8,

    ser: &'a mut DynamicSerializer,
}

pub struct PngInfo {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
}

impl<'a> PngEncoder<'a> {
    pub fn new(ser: &'a mut DynamicSerializer, header: &'a PngInfo, planes: u8) -> Self {
        let mut this = Self {
            header,
            planes,
            ser,
        };

        this.ser.write_bytes(MAGIC);
        this.write_chunk(b"IHDR", |ser| this.header.serialize(ser));
        this
    }

    fn write_chunk(&mut self, chunk: &[u8], callback: impl Fn(&mut DynamicSerializer)) {
        let length = self.ser.reserve(4);
        self.ser.write_bytes(&chunk[0..4]);

        let start = self.ser.pos();
        callback(self.ser);
        let delta = self.ser.pos() - start;
        self.ser
            .execute_at(length, |ser| ser.write_u32_be(delta as u32));

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&chunk[0..4]);
        hasher.update(self.ser.view_mut(start, delta));
        self.ser.write_u32_be(hasher.finalize());
    }

    pub fn write_pixel_dimensions(&mut self, x: u32, y: u32) {
        self.write_chunk(b"pHYs", |ser| {
            ser.write_u32_be(x);
            ser.write_u32_be(y);
            ser.write_u8(0);
        });
    }

    pub fn write_image_data(&mut self, mut rgb: Vec<Run>) {
        let width = self.header.width as u64 * self.planes as u64;
        intersperse_runs(&mut rgb, 0, width);

        let mut check = Adler32::new();
        rgb.iter().for_each(|run| check.update_run(run));
        let check = check.finish();

        let tokens = lz77_compress(rgb.into_iter());
        self.write_chunk(b"IDAT", |ser| {
            let bytes = ser.inner_mut();
            let mut bits = BitVec::new(bytes, 8);
            huffman(&mut bits, &tokens);
            ser.write_u32_be(check);
        });
    }

    pub fn write_end(&mut self) {
        self.write_chunk(b"IEND", |_| {});
    }
}

impl PngInfo {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_be(self.width);
        ser.write_u32_be(self.height);
        ser.write_u8(self.bit_depth);
        ser.write_u8(self.color_type);
        ser.write_u8(0);
        ser.write_u8(0);
        ser.write_u8(0);
    }
}

pub fn intersperse_runs(runs: &mut Vec<Run>, value: u8, spacing: u64) {
    let mut i = 0; // The current run being processed
    let mut pos = 0; // The current position in bytes
    let mut next = 0; // Next byte index to insert `value`

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
            if run.value == value {
                let n = 1 + (end - next - 1) / spacing;
                pos += run.length;
                next += spacing * n;
                run.length += n;
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

                runs.insert(i, Run { length: 1, value });
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
