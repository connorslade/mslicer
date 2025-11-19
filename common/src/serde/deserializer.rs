use std::mem;

use super::SizedString;

pub struct Deserializer<'a> {
    buffer: &'a [u8],
    offset: usize,
}

#[allow(dead_code)]
impl<'a> Deserializer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            buffer: data,
            offset: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.offset
    }

    pub fn advance_by(&mut self, amount: usize) {
        self.offset += amount;
    }

    pub fn advance<T>(&mut self) {
        self.advance_by(mem::size_of::<T>());
    }

    pub fn jump_to(&mut self, pos: usize) {
        self.offset = pos;
    }

    pub fn execute_at<T>(&mut self, pos: usize, func: impl FnOnce(&mut Self) -> T) -> T {
        let offset = self.offset;
        self.jump_to(pos);
        let result = func(self);
        self.offset = offset;
        result
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }

    pub fn read_u8(&mut self) -> u8 {
        let value = self.buffer[self.offset];
        self.offset += 1;
        value
    }

    pub fn read_u16_be(&mut self) -> u16 {
        let value = u16::from_be_bytes([self.buffer[self.offset], self.buffer[self.offset + 1]]);
        self.offset += 2;
        value
    }

    pub fn read_u16_le(&mut self) -> u16 {
        let value = u16::from_le_bytes([self.buffer[self.offset], self.buffer[self.offset + 1]]);
        self.offset += 2;
        value
    }

    pub fn read_u32_be(&mut self) -> u32 {
        let value = u32::from_be_bytes([
            self.buffer[self.offset],
            self.buffer[self.offset + 1],
            self.buffer[self.offset + 2],
            self.buffer[self.offset + 3],
        ]);
        self.offset += 4;
        value
    }

    pub fn read_u32_le(&mut self) -> u32 {
        let value = u32::from_le_bytes([
            self.buffer[self.offset],
            self.buffer[self.offset + 1],
            self.buffer[self.offset + 2],
            self.buffer[self.offset + 3],
        ]);
        self.offset += 4;
        value
    }

    pub fn read_u64_be(&mut self) -> u64 {
        let value = u64::from_be_bytes([
            self.buffer[self.offset],
            self.buffer[self.offset + 1],
            self.buffer[self.offset + 2],
            self.buffer[self.offset + 3],
            self.buffer[self.offset + 4],
            self.buffer[self.offset + 5],
            self.buffer[self.offset + 6],
            self.buffer[self.offset + 7],
        ]);
        self.offset += 8;
        value
    }

    pub fn read_u64_le(&mut self) -> u64 {
        let value = u64::from_le_bytes([
            self.buffer[self.offset],
            self.buffer[self.offset + 1],
            self.buffer[self.offset + 2],
            self.buffer[self.offset + 3],
            self.buffer[self.offset + 4],
            self.buffer[self.offset + 5],
            self.buffer[self.offset + 6],
            self.buffer[self.offset + 7],
        ]);
        self.offset += 8;
        value
    }

    pub fn read_f32_be(&mut self) -> f32 {
        let value = f32::from_be_bytes([
            self.buffer[self.offset],
            self.buffer[self.offset + 1],
            self.buffer[self.offset + 2],
            self.buffer[self.offset + 3],
        ]);
        self.offset += 4;
        value
    }

    pub fn read_f32_le(&mut self) -> f32 {
        let value = f32::from_le_bytes([
            self.buffer[self.offset],
            self.buffer[self.offset + 1],
            self.buffer[self.offset + 2],
            self.buffer[self.offset + 3],
        ]);
        self.offset += 4;
        value
    }

    pub fn read_bytes(&mut self, length: usize) -> &'a [u8] {
        let value = &self.buffer[self.offset..self.offset + length];
        self.offset += length;
        value
    }

    pub fn read_sized_string<const SIZE: usize>(&mut self) -> SizedString<SIZE> {
        SizedString::new(self.read_bytes(SIZE))
    }

    pub fn is_empty(&self) -> bool {
        self.offset == self.buffer.len()
    }
}
