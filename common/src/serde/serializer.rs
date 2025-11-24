use std::iter::repeat_n;

use super::SizedString;

pub trait Serializer {
    fn pos(&self) -> usize;
    fn write_bool(&mut self, data: bool);
    fn write_u8(&mut self, data: u8);
    fn write_u16_be(&mut self, data: u16);
    fn write_u16_le(&mut self, data: u16);
    fn write_u32_be(&mut self, data: u32);
    fn write_u32_le(&mut self, data: u32);
    fn write_u64_be(&mut self, data: u64);
    fn write_u64_le(&mut self, data: u64);
    fn write_f32_be(&mut self, data: f32);
    fn write_f32_le(&mut self, data: f32);
    fn write_f64_be(&mut self, data: f64);
    fn write_f64_le(&mut self, data: f64);
    fn write_bytes(&mut self, data: &[u8]);
    fn reserve(&mut self, length: usize) -> usize;
    fn execute_at(&mut self, offset: usize, f: impl FnOnce(&mut SizedSerializer));
    fn view_mut(&mut self, offset: usize, size: usize) -> &mut [u8];
    fn write_sized_string<const SIZE: usize>(&mut self, data: &SizedString<SIZE>);
}

pub struct SizedSerializer<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

pub struct DynamicSerializer {
    buffer: Vec<u8>,
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

impl Serializer for SizedSerializer<'_> {
    fn pos(&self) -> usize {
        self.offset
    }

    fn write_bool(&mut self, data: bool) {
        self.write_u8(data as u8);
    }

    fn write_u8(&mut self, data: u8) {
        self.buffer[self.offset] = data;
        self.offset += 1;
    }

    fn write_u16_be(&mut self, data: u16) {
        self.buffer[self.offset..self.offset + 2].copy_from_slice(&data.to_be_bytes());
        self.offset += 2;
    }

    fn write_u16_le(&mut self, data: u16) {
        self.buffer[self.offset..self.offset + 2].copy_from_slice(&data.to_le_bytes());
        self.offset += 2;
    }

    fn write_u32_be(&mut self, data: u32) {
        self.buffer[self.offset..self.offset + 4].copy_from_slice(&data.to_be_bytes());
        self.offset += 4;
    }

    fn write_u32_le(&mut self, data: u32) {
        self.buffer[self.offset..self.offset + 4].copy_from_slice(&data.to_le_bytes());
        self.offset += 4;
    }

    fn write_u64_be(&mut self, data: u64) {
        self.buffer[self.offset..self.offset + 8].copy_from_slice(&data.to_be_bytes());
        self.offset += 8;
    }

    fn write_u64_le(&mut self, data: u64) {
        self.buffer[self.offset..self.offset + 8].copy_from_slice(&data.to_le_bytes());
        self.offset += 8;
    }

    fn write_f32_be(&mut self, data: f32) {
        self.buffer[self.offset..self.offset + 4].copy_from_slice(&data.to_be_bytes());
        self.offset += 4;
    }

    fn write_f32_le(&mut self, data: f32) {
        self.buffer[self.offset..self.offset + 4].copy_from_slice(&data.to_le_bytes());
        self.offset += 4;
    }

    fn write_f64_be(&mut self, data: f64) {
        self.buffer[self.offset..self.offset + 8].copy_from_slice(&data.to_be_bytes());
        self.offset += 8;
    }

    fn write_f64_le(&mut self, data: f64) {
        self.buffer[self.offset..self.offset + 8].copy_from_slice(&data.to_le_bytes());
        self.offset += 8;
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

    fn write_sized_string<const SIZE: usize>(&mut self, data: &SizedString<SIZE>) {
        let len = data.data.len();
        self.buffer[self.offset..self.offset + len].copy_from_slice(&data.data);
        self.offset += len;
    }
}

impl Serializer for DynamicSerializer {
    fn pos(&self) -> usize {
        self.buffer.len()
    }

    fn write_bool(&mut self, data: bool) {
        self.write_u8(data as u8);
    }

    fn write_u8(&mut self, data: u8) {
        self.buffer.push(data);
    }

    fn write_u16_be(&mut self, data: u16) {
        self.buffer.extend_from_slice(&data.to_be_bytes());
    }

    fn write_u16_le(&mut self, data: u16) {
        self.buffer.extend_from_slice(&data.to_le_bytes());
    }

    fn write_u32_be(&mut self, data: u32) {
        self.buffer.extend_from_slice(&data.to_be_bytes());
    }

    fn write_u32_le(&mut self, data: u32) {
        self.buffer.extend_from_slice(&data.to_le_bytes());
    }

    fn write_u64_be(&mut self, data: u64) {
        self.buffer.extend_from_slice(&data.to_be_bytes());
    }

    fn write_u64_le(&mut self, data: u64) {
        self.buffer.extend_from_slice(&data.to_le_bytes());
    }

    fn write_f32_be(&mut self, data: f32) {
        self.buffer.extend_from_slice(&data.to_be_bytes());
    }

    fn write_f32_le(&mut self, data: f32) {
        self.buffer.extend_from_slice(&data.to_le_bytes());
    }

    fn write_f64_be(&mut self, data: f64) {
        self.buffer.extend_from_slice(&data.to_be_bytes());
    }

    fn write_f64_le(&mut self, data: f64) {
        self.buffer.extend_from_slice(&data.to_le_bytes());
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

    fn write_sized_string<const SIZE: usize>(&mut self, data: &SizedString<SIZE>) {
        self.buffer.extend_from_slice(&data.data);
    }
}

impl Default for DynamicSerializer {
    fn default() -> Self {
        Self::new()
    }
}
