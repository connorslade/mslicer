/// Wraps a byte vector and allows building it bit by bit.
pub struct BitVec<'a> {
    bytes: &'a mut Vec<u8>,
    index: usize,
}

impl<'a> BitVec<'a> {
    pub fn new(bytes: &'a mut Vec<u8>, bits: usize) -> Self {
        Self {
            index: (bytes.len().saturating_sub(1)) * 8 + bits,
            bytes,
        }
    }

    pub fn push(&mut self, value: bool) {
        let bit_index = self.index % 8;
        (bit_index == 0).then(|| self.bytes.push(0));

        self.index += 1;
        let byte = self.bytes.last_mut().unwrap();
        *byte |= (value as u8) << bit_index;
    }

    pub fn finish_byte(&mut self) {
        self.index = self.index.next_multiple_of(8);
    }

    pub fn extend(&mut self, val: u32, bits: u8) {
        (0..bits).for_each(|bit| self.push((val >> bit) & 0b1 != 0));
    }

    pub fn extend_rev(&mut self, val: u32, bits: u8) {
        ((0..bits).rev()).for_each(|bit| self.push((val >> bit) & 0b1 != 0));
    }
}
