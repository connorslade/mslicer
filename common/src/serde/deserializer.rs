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

    pub fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }

    pub fn read_u8(&mut self) -> u8 {
        let value = self.buffer[self.offset];
        self.offset += 1;
        value
    }

    pub fn read_u16(&mut self) -> u16 {
        let value = u16::from_be_bytes([self.buffer[self.offset], self.buffer[self.offset + 1]]);
        self.offset += 2;
        value
    }

    pub fn read_u32(&mut self) -> u32 {
        let value = u32::from_be_bytes([
            self.buffer[self.offset],
            self.buffer[self.offset + 1],
            self.buffer[self.offset + 2],
            self.buffer[self.offset + 3],
        ]);
        self.offset += 4;
        value
    }

    pub fn read_u64(&mut self) -> u64 {
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

    pub fn read_f32(&mut self) -> f32 {
        let value = f32::from_be_bytes([
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
