//! Custom PNG encoder optimized for encoding RLE byte streams.
//!
//! Reference: https://www.bamfordresearch.com/2021/one-hour-png

use std::mem;

use common::serde::{DynamicSerializer, Serializer};

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

        let mut ser = DynamicSerializer::new();
        let mut zlib = ZlibStore::new(&mut ser);
        zlib.write_data(&blob);
        zlib.finish();

        self.write_chunk(b"IDAT", &ser.into_inner());
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

// Reference: https://github.com/image-rs/fdeflate/blob/c365c7e6ffa81feb2e1fb762eed7299f05c9b0ca/src/compress.rs
struct ZlibStore<'a, T: Serializer> {
    ser: &'a mut T,

    check: Adler32,
    block_bytes: u16,
    block_header: Option<usize>,
}

impl<'a, T: Serializer> ZlibStore<'a, T> {
    fn new(ser: &'a mut T) -> Self {
        ser.write_bytes(&[0x78, 0x01]);
        let block_header = Some(ser.reserve(5));

        Self {
            ser,

            check: Adler32::default(),
            block_bytes: 0,
            block_header,
        }
    }

    fn set_block_header(&mut self, size: u16, last: bool) {
        if let Some(header) = mem::take(&mut self.block_header) {
            self.ser.execute_at(header, |ser| {
                // todo: cleanup
                ser.write_bytes(&[
                    last as u8,
                    (size & 0xFF) as u8,
                    ((size >> 8) & 0xFF) as u8,
                    (!size & 0xFF) as u8,
                    ((!size >> 8) & 0xFF) as u8,
                ]);
            });
        }
    }

    fn write_data(&mut self, mut data: &[u8]) {
        self.check.update(data);
        while !data.is_empty() {
            if self.block_bytes == u16::MAX {
                self.set_block_header(u16::MAX, false);
                self.block_header = Some(self.ser.reserve(5));
                self.block_bytes = 0;
            }

            let prefix_bytes = data.len().min((u16::MAX - self.block_bytes) as usize);
            self.ser.write_bytes(&data[..prefix_bytes]);
            self.block_bytes += prefix_bytes as u16;
            data = &data[prefix_bytes..];
        }
    }

    pub fn finish(mut self) {
        self.set_block_header(self.block_bytes, true);

        let checksum = self.check.finish();
        self.ser.write_u32_be(checksum);
    }
}

#[derive(Default)]
struct Adler32 {
    a: u16,
    b: u16,
}

impl Adler32 {
    pub fn update(&mut self, data: &[u8]) {
        const MOD: u32 = 65521;
        const NMAX: usize = 5552;

        let (mut a, mut b) = (self.a as u32, self.b as u32);
        for chunk in data.chunks(NMAX) {
            for byte in chunk {
                a = a.wrapping_add(*byte as _);
                b = b.wrapping_add(a);
            }

            a %= MOD;
            b %= MOD;
        }

        self.a = a as u16;
        self.b = b as u16;
    }

    pub fn finish(self) -> u32 {
        (self.b as u32) << 16 | self.a as u32
    }
}
