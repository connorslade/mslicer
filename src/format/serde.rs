pub struct Serializer<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

pub struct SizedString<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<'a> Serializer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, offset: 0 }
    }

    pub fn write_bool(&mut self, data: bool) {
        self.write_u8(data as u8);
    }

    pub fn write_u8(&mut self, data: u8) {
        self.buffer[self.offset] = data;
        self.offset += 1;
    }

    pub fn write_u16(&mut self, data: u16) {
        self.buffer[self.offset..self.offset + 2].copy_from_slice(&data.to_be_bytes());
        self.offset += 2;
    }

    pub fn write_u32(&mut self, data: u32) {
        self.buffer[self.offset..self.offset + 4].copy_from_slice(&data.to_be_bytes());
        self.offset += 4;
    }

    pub fn write_u64(&mut self, data: u64) {
        self.buffer[self.offset..self.offset + 8].copy_from_slice(&data.to_be_bytes());
        self.offset += 8;
    }

    pub fn write_f32(&mut self, data: f32) {
        self.buffer[self.offset..self.offset + 4].copy_from_slice(&data.to_be_bytes());
        self.offset += 4;
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buffer[self.offset..self.offset + data.len()].copy_from_slice(data);
        self.offset += data.len();
    }

    pub fn write_sized_string<const SIZE: usize>(&mut self, data: &SizedString<SIZE>) {
        let len = data.data.len();
        self.buffer[self.offset..self.offset + len].copy_from_slice(&data.data);
        self.offset += len;
    }
}

impl<const SIZE: usize> SizedString<SIZE> {
    pub const fn new(data: &[u8]) -> Self {
        debug_assert!(data.len() <= SIZE);

        // kinda crazy this works in a const fn
        let mut arr = [0; SIZE];
        let mut i = 0;
        while i < SIZE && i < data.len() {
            arr[i] = data[i];
            i += 1;
        }

        Self { data: arr }
    }
}
