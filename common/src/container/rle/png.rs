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

use crate::{
    container::{BitVec, Run},
    serde::{DynamicSerializer, Serializer},
};

use deflate::{Adler32, huffman, lz77_compress};
use nalgebra::Vector2;

const MAGIC: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];

pub struct PngEncoder<'a> {
    size: Vector2<u32>,
    color: ColorType,

    ser: &'a mut DynamicSerializer,
}

#[derive(Clone, Copy)]
pub enum ColorType {
    Grayscale = 0,
    Truecolor = 2,
}

impl<'a> PngEncoder<'a> {
    pub fn new(ser: &'a mut DynamicSerializer, color: ColorType, size: Vector2<u32>) -> Self {
        let mut this = Self { size, color, ser };

        this.ser.write_bytes(MAGIC);
        this.write_chunk(b"IHDR", |ser| {
            ser.write_u32_be(size.x);
            ser.write_u32_be(size.y);
            ser.write_u8(8); // 8 bits per pixel per channel
            ser.write_u8(color as u8);
            ser.write_u8(0); // compression
            ser.write_u8(0); // filter
            ser.write_u8(0); // interlace
        });
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

    pub fn write_image_data(&mut self, mut data: Vec<Run>) {
        let width = self.size.x as u64 * self.color.planes() as u64;
        intersperse_runs(&mut data, 0, width);

        let mut check = Adler32::new();
        data.iter().for_each(|run| check.update_run(run));
        let check = check.finish();

        let tokens = lz77_compress(data.into_iter());
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

impl ColorType {
    fn planes(&self) -> u8 {
        match self {
            ColorType::Grayscale => 1,
            ColorType::Truecolor => 3,
        }
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

pub mod deflate {
    use crate::container::{BitVec, Run};

    pub struct Adler32 {
        a: u16,
        b: u16,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum LZ77Token {
        Literal(u8),
        Match { length: u16 }, // distance is assumed to be 1
    }

    /// Assumes all input runs are of nonzero length.
    pub fn lz77_compress(runs: impl Iterator<Item = Run>) -> Vec<LZ77Token> {
        let mut out = Vec::new();

        for run in runs {
            debug_assert!(run.length > 0);

            out.push(LZ77Token::Literal(run.value));
            let mut remaining = run.length.saturating_sub(1);

            while remaining >= 3 {
                let match_len = remaining.min(258);
                remaining -= match_len;
                out.push(LZ77Token::Match {
                    length: match_len as u16,
                });
            }

            out.extend((0..remaining).map(|_| LZ77Token::Literal(run.value)));
        }

        out
    }

    pub fn huffman(out: &mut BitVec, tokens: &[LZ77Token]) {
        out.extend(0x78, 8);
        out.extend(0x01, 8);
        out.extend(0b011, 3);

        for token in tokens {
            match token {
                LZ77Token::Literal(val) => huffman_code(out, *val as u32),
                LZ77Token::Match { length } => {
                    let (code, ebits, nbits) = length_code(*length);
                    huffman_code(out, code as u32);
                    (nbits >= 1).then(|| out.extend(ebits as u32, nbits));

                    // For a more general encoder, we would have to get the distance
                    // code + extra bits. But since we are only handling Matches
                    // with a distance of 1, we can inline the values you would get.
                    out.advance(5);
                }
            }
        }

        huffman_code(out, 256);
    }

    impl Adler32 {
        const MOD: u32 = 65521;

        pub fn new() -> Self {
            Self { a: 1, b: 0 }
        }

        // Efficiently update the checksum state for `length` bytes with value
        // `value`. This is done in batches of 380,368,439 to avoid the possibility
        // of overflows. This constant was derived by finding the largest integers
        // that satisfies the following equation. Note that 65520 is the maximum
        // value of `a` and 255 is the maximum value of `value`.
        //
        // (l * 65520) + (255 * (l * (l + 1) / 2)) < 2^64 - 1
        pub fn update_run(&mut self, run: &Run) {
            let (mut total_length, value) = (run.length, run.value as u64);
            while total_length > 0 {
                let length = total_length.min(380_368_439);
                total_length -= length;

                let (mut a, mut b) = (self.a as u64, self.b as u64);
                b += (length * a) + (value * (length * (length + 1) / 2));
                a += length * value;

                self.a = (a % Self::MOD as u64) as u16;
                self.b = (b % Self::MOD as u64) as u16;
            }
        }

        pub fn finish(self) -> u32 {
            (self.b as u32) << 16 | self.a as u32
        }
    }

    impl Default for Adler32 {
        fn default() -> Self {
            Self::new()
        }
    }

    fn huffman_code(vec: &mut BitVec, val: u32) {
        match val {
            0..=143 => vec.extend_rev(val + 0x30, 8),
            144..=255 => vec.extend_rev(val - 144 + 0x190, 9),
            256..=279 => vec.extend_rev(val - 256, 7),
            280..=287 => vec.extend_rev(val - 280 + 0xC0, 8),
            _ => unreachable!(),
        }
    }

    fn length_code(n: u16) -> (u16, u16, u8) {
        match n {
            3..=10 => (254 + n, 0, 0),
            11..=18 => (265 + (n - 11) / 2, (n - 11) % 2, 1),
            19..=34 => (269 + (n - 19) / 4, (n - 19) % 4, 2),
            35..=66 => (273 + (n - 35) / 8, (n - 35) % 8, 3),
            67..=130 => (277 + (n - 67) / 16, (n - 67) % 16, 4),
            131..258 => (281 + (n - 131) / 32, (n - 131) % 32, 5),
            258 => (285, 0, 0),
            _ => unreachable!(),
        }
    }
}
