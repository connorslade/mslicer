#[derive(Default)]
pub struct BitVec {
    bytes: Vec<u8>,
    index: usize,
}

impl BitVec {
    pub fn from_raw(bytes: Vec<u8>, bits: usize) -> Self {
        Self {
            index: (bytes.len().saturating_sub(1)) * 8 + bits,
            bytes,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.bytes
    }

    pub fn push(&mut self, value: bool) {
        let bit_index = self.index % 8;
        if bit_index == 0 {
            self.bytes.push(0);
        }
        self.index += 1;

        let byte = self.bytes.last_mut().unwrap();
        *byte |= (value as u8) << bit_index;
    }

    pub fn extend(&mut self, val: u32, bits: u8) {
        (0..bits).for_each(|bit| self.push((val >> bit) & 0b1 != 0));
    }

    pub fn extend_rev(&mut self, val: u32, bits: u8) {
        ((0..bits).rev()).for_each(|bit| self.push((val >> bit) & 0b1 != 0));
    }
}
