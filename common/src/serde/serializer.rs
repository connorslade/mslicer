use std::{
    io::{Seek, Write},
    iter::repeat_n,
};

#[rustfmt::skip]
pub trait Serializer {
    fn pos(&mut self) -> usize;
    fn write_bytes(&mut self, data: &[u8]);
    fn reserve(&mut self, length: usize) -> usize;
    fn execute_at(&mut self, offset: usize, f: impl FnOnce(&mut SizedSerializer));
    fn view_mut(&mut self, offset: usize, size: usize) -> &mut [u8];

    fn write_bool(&mut self, data: bool) { self.write_u8(data as u8); }
    fn write_u8(&mut self, data: u8) { self.write_bytes(&[data]); }
    fn write_u16_be(&mut self, data: u16) { self.write_bytes(&data.to_be_bytes()); }
    fn write_u16_le(&mut self, data: u16) { self.write_bytes(&data.to_le_bytes()); }
    fn write_u32_be(&mut self, data: u32) { self.write_bytes(&data.to_be_bytes()); }
    fn write_u32_le(&mut self, data: u32) { self.write_bytes(&data.to_le_bytes()); }
    fn write_u64_be(&mut self, data: u64) { self.write_bytes(&data.to_be_bytes()); }
    fn write_u64_le(&mut self, data: u64) { self.write_bytes(&data.to_le_bytes()); }
    fn write_f32_be(&mut self, data: f32) { self.write_bytes(&data.to_be_bytes()); }
    fn write_f32_le(&mut self, data: f32) { self.write_bytes(&data.to_le_bytes()); }
    fn write_f64_be(&mut self, data: f64) { self.write_bytes(&data.to_be_bytes()); }
    fn write_f64_le(&mut self, data: f64) { self.write_bytes(&data.to_le_bytes()); }
}

pub struct SizedSerializer<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

pub struct DynamicSerializer {
    buffer: Vec<u8>,
}

pub struct WriterSerializer<T: Write + Seek> {
    stream: T,
}

impl<'a> SizedSerializer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, offset: 0 }
    }
}

impl DynamicSerializer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buffer
    }
}

impl<T: Write + Seek> WriterSerializer<T> {
    pub fn new(stream: T) -> Self {
        Self { stream }
    }
}

impl Serializer for SizedSerializer<'_> {
    fn pos(&mut self) -> usize {
        self.offset
    }

    fn write_bytes(&mut self, data: &[u8]) {
        self.buffer[self.offset..self.offset + data.len()].copy_from_slice(data);
        self.offset += data.len();
    }

    fn reserve(&mut self, length: usize) -> usize {
        let out = self.offset;
        self.offset += length;
        out
    }

    fn execute_at(&mut self, offset: usize, f: impl FnOnce(&mut SizedSerializer)) {
        let mut ser = SizedSerializer::new(&mut self.buffer[offset..]);
        f(&mut ser);
    }

    fn view_mut(&mut self, offset: usize, size: usize) -> &mut [u8] {
        &mut self.buffer[offset..(offset + size)]
    }
}

impl Serializer for DynamicSerializer {
    fn pos(&mut self) -> usize {
        self.buffer.len()
    }

    fn write_bytes(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    fn reserve(&mut self, length: usize) -> usize {
        let start = self.buffer.len();
        self.buffer.extend(repeat_n(0, length));
        start
    }

    fn execute_at(&mut self, offset: usize, f: impl FnOnce(&mut SizedSerializer)) {
        let mut ser = SizedSerializer::new(&mut self.buffer[offset..]);
        f(&mut ser);
    }

    fn view_mut(&mut self, offset: usize, size: usize) -> &mut [u8] {
        &mut self.buffer[offset..(offset + size)]
    }
}

impl<T: Write + Seek> Serializer for WriterSerializer<T> {
    fn pos(&mut self) -> usize {
        self.stream.stream_position().unwrap() as usize
    }

    fn write_bytes(&mut self, data: &[u8]) {
        self.stream.write_all(data).unwrap();
    }

    fn reserve(&mut self, _length: usize) -> usize {
        unimplemented!()
    }

    fn execute_at(&mut self, _offset: usize, _f: impl FnOnce(&mut SizedSerializer)) {
        unimplemented!()
    }

    fn view_mut(&mut self, _offset: usize, _size: usize) -> &mut [u8] {
        unimplemented!()
    }
}

impl Default for DynamicSerializer {
    fn default() -> Self {
        Self::new()
    }
}
