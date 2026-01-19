use std::fmt::{self, Debug, Display};

use nalgebra::{ArrayStorage, Const, Matrix};

use crate::serde::{Deserializer, Serializer};

pub trait SerdeExt {
    fn serialize<T: Serializer>(&self, ser: &mut T);
    fn deserialize<T: Deserializer>(des: &mut T) -> Self;
}

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

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        SizedString::new(&des.read_bytes(SIZE))
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_bytes(&self.data);
    }
}

impl<const SIZE: usize> Display for SizedString<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let null = self.data.iter().position(|&x| x == 0).unwrap_or(32);
        f.write_str(&String::from_utf8_lossy(&self.data[..null]))
    }
}

impl<const SIZE: usize> Debug for SizedString<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let null = self.data.iter().position(|&x| x == 0).unwrap_or(SIZE);
        let str = String::from_utf8_lossy(&self.data[..null]);
        f.write_fmt(format_args!("{str:?}"))
    }
}

impl<const N: usize> SerdeExt for Matrix<f32, Const<N>, Const<1>, ArrayStorage<f32, N, 1>>
where
    ArrayStorage<f32, N, 1>: Default,
{
    fn serialize<T: Serializer>(&self, ser: &mut T) {
        for i in 0..N {
            ser.write_f32_be(self[i]);
        }
    }

    fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        let mut out = Self::default();
        for i in 0..N {
            out[i] = des.read_f32_be();
        }
        out
    }
}

impl<const N: usize> SerdeExt for Matrix<u32, Const<N>, Const<1>, ArrayStorage<u32, N, 1>>
where
    ArrayStorage<u32, N, 1>: Default,
{
    fn serialize<T: Serializer>(&self, ser: &mut T) {
        for i in 0..N {
            ser.write_u32_be(self[i]);
        }
    }

    fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        let mut out = Self::default();
        for i in 0..N {
            out[i] = des.read_u32_be();
        }
        out
    }
}
