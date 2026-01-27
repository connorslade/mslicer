//! Custom PNG encoder optimized for encoding RLE byte streams.
//!
//! ## References
//! - https://www.bamfordresearch.com/2021/one-hour-png
//! - https://www.rfc-editor.org/rfc/rfc1951
//! - https://github.com/image-rs/fdeflate/blob/c365c7e6ffa81feb2e1fb762eed7299f05c9b0ca/src/compress.rs
//! - https://gist.github.com/juliusgeo/d4965b16a3c4478bb4eca2fe210559eb

use std::mem;

use common::{
    bitvec::BitVec,
    misc::Run,
    serde::{DynamicSerializer, Serializer},
};

const MAGIC: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];

pub struct PngEncoder<'a, T: Serializer> {
    header: &'a PngInfo,
    planes: u8,

    ser: &'a mut T,
}

pub struct PngInfo {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
}

impl<'a, T: Serializer> PngEncoder<'a, T> {
    pub fn new(ser: &'a mut T, header: &'a PngInfo, planes: u8) -> Self {
        let mut this = Self {
            header,
            planes,
            ser,
        };

        this.ser.write_bytes(MAGIC);
        this.write_chunk(b"IHDR", &this.header.serialize());
        this
    }

    fn write_chunk(&mut self, chunk: &[u8], data: &[u8]) {
        self.ser.write_u32_be(data.len() as u32);
        self.ser.write_bytes(&chunk[0..4]);
        self.ser.write_bytes(data);

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&chunk[0..4]);
        hasher.update(data);
        self.ser.write_u32_be(hasher.finalize());
    }

    pub fn write_image(&mut self, rgb: &[u8]) {
        let mut blob = Vec::new();

        let width = self.header.width as usize * self.planes as usize;
        for y in 0..self.header.height as usize {
            blob.push(0);
            blob.extend_from_slice(&rgb[(y * width)..((y + 1) * width)]);
        }

        let runs = run_length_encode(&blob);
        // println!("{runs:?}");

        let mut check = Adler32 { a: 1, b: 0 };
        check.update(&blob);

        let tokens = lz77_compress(runs.into_iter());
        // dbg!(&tokens);
        let mut bytes = tokens_to_stream(&tokens);
        bytes.extend_from_slice(&check.finish().to_be_bytes());
        // dbg!(bytes.len());
        self.write_chunk(b"IDAT", &bytes);
    }

    pub fn write_end(&mut self) {
        self.write_chunk(b"IEND", &[]);
    }
}

impl PngInfo {
    pub fn serialize(&self) -> Vec<u8> {
        let mut ser = DynamicSerializer::new();
        ser.write_u32_be(self.width);
        ser.write_u32_be(self.height);
        ser.write_u8(self.bit_depth);
        ser.write_u8(self.color_type);
        ser.write_u8(0);
        ser.write_u8(0);
        ser.write_u8(0);
        ser.into_inner()
    }
}

fn run_length_encode(data: &[u8]) -> Vec<Run> {
    let mut runs = Vec::new();

    let mut last_byte = data[0];
    let mut length = 0;

    for &byte in data {
        if last_byte != byte {
            if length > 0 {
                runs.push(Run {
                    length: mem::take(&mut length),
                    value: last_byte,
                });
            }
            last_byte = byte;
        }

        length += 1;
    }

    if length > 0 {
        runs.push(Run {
            length,
            value: last_byte,
        });
    }

    runs
}

#[derive(Default)]
struct Adler32 {
    a: u16,
    b: u16,
}

impl Adler32 {
    const MOD: u32 = 65521;
    const NMAX: usize = 5552;

    pub fn update(&mut self, data: &[u8]) {
        let (mut a, mut b) = (self.a as u32, self.b as u32);
        for chunk in data.chunks(Self::NMAX) {
            for byte in chunk {
                a = a.wrapping_add(*byte as _);
                b = b.wrapping_add(a);
            }

            a %= Self::MOD;
            b %= Self::MOD;
        }

        self.a = a as u16;
        self.b = b as u16;
    }

    pub fn finish(self) -> u32 {
        (self.b as u32) << 16 | self.a as u32
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LZ77Token {
    Literal(u8),
    Match { distance: u16, length: u16 },
}

fn lz77_compress(rle: impl Iterator<Item = Run>) -> Vec<LZ77Token> {
    let mut out = Vec::new();

    for run in rle {
        out.push(LZ77Token::Literal(run.value));
        let mut remaining = run.length.saturating_sub(1);

        while remaining >= 3 {
            let match_len = remaining.min(258);
            remaining -= match_len;
            out.push(LZ77Token::Match {
                distance: 1,
                length: match_len as u16,
            });
        }

        out.extend((0..remaining).map(|_| LZ77Token::Literal(run.value)));
    }

    out
}

fn huffman_code(vec: &mut BitVec, val: u32) {
    match val {
        0..144 => vec.extend_rev(val + 0x30, 8),
        144..256 => vec.extend_rev(val - 144 + 0x190, 9),
        256..280 => vec.extend_rev(val - 256, 7),
        280..288 => vec.extend_rev(val - 280 + 0xC0, 8),
        _ => panic!(),
    }
}

fn length_code(n: u16) -> (u16, u16, u8) {
    match n {
        0..=2 => (n, 0, 0),
        3..=10 => (254 + n, 0, 0),
        11..=18 => (265 + (n - 11) / 2, (n - 11) % 2, 1),
        19..=34 => (269 + (n - 19) / 4, (n - 19) % 4, 2),
        35..=66 => (273 + (n - 35) / 8, (n - 35) % 8, 3),
        67..=130 => (277 + (n - 67) / 16, (n - 67) % 16, 4),
        131..258 => (281 + (n - 131) / 32, (n - 131) % 32, 5),
        258 => (285, 0, 0),
        _ => panic!(),
    }
}

fn distance_code(n: u16) -> (u16, u16, u8) {
    match n {
        0..=4 => (n - 1, 0, 0),
        5..=8 => ((n - 5) / 2 + 4, (n - 5), 1),
        9..=16 => ((n - 9) / 4 + 6, (n - 9), 2),
        17..=32 => ((n - 17) / 8 + 8, (n - 17), 3),
        33..=64 => ((n - 33) / 16 + 10, (n - 33), 4),
        65..=128 => ((n - 65) / 32 + 12, (n - 65), 5),
        129..=256 => ((n - 129) / 64 + 14, (n - 129), 6),
        257..=512 => ((n - 257) / 128 + 16, (n - 257), 7),
        513..=1024 => ((n - 513) / 256 + 18, (n - 513), 8),
        1025..=2048 => ((n - 1025) / 512 + 20, (n - 1025), 9),
        2049..=4096 => ((n - 2049) / 1024 + 22, (n - 2049), 10),
        4097..=8192 => ((n - 4097) / 2048 + 24, (n - 4097), 11),
        8193..=16384 => ((n - 8193) / 4096 + 26, (n - 8193), 12),
        16385..=32768 => ((n - 16385) / 8192 + 28, (n - 16385), 13),
        _ => panic!(),
    }
}

fn tokens_to_stream(tokens: &[LZ77Token]) -> Vec<u8> {
    let mut out = BitVec::from_raw(vec![0x78, 0x01], 8);
    out.extend(0b011, 3);

    for token in tokens {
        match token {
            LZ77Token::Literal(val) => huffman_code(&mut out, *val as u32),
            LZ77Token::Match { distance, length } => {
                let (code, ebits, nbits) = length_code(*length);
                huffman_code(&mut out, code as u32);
                (nbits >= 1).then(|| out.extend(ebits as u32, nbits));

                let (code, ebits, nbits) = distance_code(*distance);
                out.extend(code as u32, 5);
                (nbits >= 1).then(|| out.extend(ebits as u32, nbits));
            }
        }
    }

    huffman_code(&mut out, 256);
    out.into_inner()
}
