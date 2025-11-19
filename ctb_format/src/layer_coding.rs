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

        let mut code = self.data[self.offset];
        let mut length = 1;

        if code & 0x80 != 0 {
            code &= 0x7F;
            self.offset += 1;

            let next = self.data[self.offset] as u64;
            if next & 0x80 == 0 {
                length = next;
            } else if next & 0xc0 == 0x80 {
                length = ((next & 0x3F) << 8) + self.data[self.offset + 1] as u64;
                self.offset += 1;
            } else if next & 0xE0 == 0xC0 {
                length = ((next & 0x1F) << 16)
                    + ((self.data[self.offset + 1] as u64) << 8)
                    + self.data[self.offset + 2] as u64;
                self.offset += 2;
            } else if next & 0xF0 == 0xE0 {
                length = ((next & 0xf) << 24)
                    + ((self.data[self.offset + 1] as u64) << 16)
                    + ((self.data[self.offset + 2] as u64) << 8)
                    + self.data[self.offset + 3] as u64;
                self.offset += 3;
            } else {
                panic!("Invalid layer data");
            }
        }

        self.offset += 1;
        if code != 0 {
            code = (code << 1) | 1;
        }

        Some(Run {
            length,
            value: code,
        })
    }
}
