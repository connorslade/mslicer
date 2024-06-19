use std::fmt::{self, Debug, Display};

pub struct SizedString<const SIZE: usize> {
    pub(crate) data: [u8; SIZE],
}

impl<const SIZE: usize> SizedString<SIZE> {
    pub const fn new_full(data: [u8; SIZE]) -> Self {
        Self { data }
    }

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

impl Display for SizedString<32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let null = self.data.iter().position(|&x| x == 0).unwrap_or(32);
        f.write_str(&String::from_utf8_lossy(&self.data[..null]))
    }
}

impl<const SIZE: usize> Debug for SizedString<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let null = self.data.iter().position(|&x| x == 0).unwrap_or(SIZE);
        f.write_fmt(format_args!(
            "\"{}\"",
            String::from_utf8_lossy(&self.data[..null])
        ))
    }
}
