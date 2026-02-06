/// Wraps a byte vector and allows building it bit by bit. Bits are ordered
/// least to most significant.
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

    /// More efficient way to push multiple bits at once. Equivalent to:
    /// ```rust
    /// (0..bits).for_each(|bit| self.push((val >> bit) & 0b1 != 0));
    /// ```
    pub fn extend(&mut self, mut val: u32, mut bits: u8) {
        let filled_bits = self.index % 8;
        self.index += bits as usize;

        // If the last byte is not full, start there.
        if filled_bits != 0 {
            let to_process = bits.min(8 - filled_bits as u8);
            let byte = self.bytes.last_mut().unwrap();
            let mask = (1 << to_process) - 1;
            *byte |= ((val & mask) << filled_bits) as u8;

            bits -= to_process;
            val >>= to_process;
        }

        // Add the remaining bits in chunks of 8 (bytes).
        while bits > 0 {
            self.bytes.push((val & 0xFF) as u8);
            val >>= 8;
            bits = bits.saturating_sub(8);
        }
    }

    pub fn extend_rev(&mut self, val: u32, bits: u8) {
        self.extend(val.reverse_bits() >> (32 - bits), bits);
    }

    // Special case of extend or extend_rev where the value is 0.
    pub fn advance(&mut self, bits: u8) {
        self.index += bits as usize;
        self.bytes.resize(self.index.div_ceil(8), 0);
    }
}
