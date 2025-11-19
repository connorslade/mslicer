use common::misc::Run;

pub struct LayerDecoder<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> LayerDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl Iterator for LayerDecoder<'_> {
    type Item = Run;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        let mut value = self.data[self.offset];
        let mut length = 1;

        if value & 0x80 != 0 {
            value &= 0x7F;
            self.offset += 1;

            let next = self.data[self.offset] as u64;
            if next & 0x80 == 0 {
                length = next;
            } else if next & 0xC0 == 0x80 {
                length = ((next & 0x3F) << 8) + self.data[self.offset + 1] as u64;
                self.offset += 1;
            } else if next & 0xE0 == 0xC0 {
                length = ((next & 0x1F) << 16)
                    + ((self.data[self.offset + 1] as u64) << 8)
                    + self.data[self.offset + 2] as u64;
                self.offset += 2;
            } else if next & 0xF0 == 0xE0 {
                length = ((next & 0xF) << 24)
                    + ((self.data[self.offset + 1] as u64) << 16)
                    + ((self.data[self.offset + 2] as u64) << 8)
                    + self.data[self.offset + 3] as u64;
                self.offset += 3;
            } else {
                self.offset = self.data.len();
                return None;
            }
        }

        self.offset += 1;
        if value != 0 {
            value = (value << 1) | 1;
        }

        Some(Run { length, value })
    }
}

#[derive(Default)]
pub struct LayerEncoder {
    data: Vec<u8>,
}

impl LayerEncoder {
    pub fn add_run(&mut self, length: u64, mut value: u8) {
        if length >= 1 {
            value |= 0x80;
        }
        self.data.push(value);

        if length <= 0x7F {
            self.data.push(length as u8);
        } else if length <= 0x3FFF {
            self.data.push((length >> 8) as u8 | 0x80);
            self.data.push(length as u8);
        } else if length <= 0x1FFFFF {
            self.data.push((length >> 16) as u8 | 0xc0);
            self.data.push((length >> 8) as u8);
            self.data.push(length as u8);
        } else if length <= 0xFFFFFFF {
            self.data.push((length >> 24) as u8 | 0xe0);
            self.data.push((length >> 16) as u8);
            self.data.push((length >> 8) as u8);
            self.data.push(length as u8);
        }
    }
}
