use std::borrow::Cow;

use anyhow::Result;
use common::serde::{Deserializer, Serializer, SliceDeserializer};

mod encoding;
mod preview;

pub(crate) use encoding::xor_cypher;
pub use {
    encoding::{LayerDecoder, LayerEncoder},
    preview::PreviewImage,
};

#[derive(Debug)]
pub struct Section {
    pub size: u32,
    pub offset: u32,
}

impl Section {
    pub fn new(offset: usize, size: usize) -> Self {
        Self {
            size: size as u32,
            offset: offset as u32,
        }
    }

    /// Offset, Size
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            offset: des.read_u32_le(),
            size: des.read_u32_le(),
        })
    }

    // Size, Offset
    pub fn deserialize_rev(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            size: des.read_u32_le(),
            offset: des.read_u32_le(),
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(self.offset);
        ser.write_u32_le(self.size);
    }

    pub fn serialize_rev<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(self.size);
        ser.write_u32_le(self.offset);
    }
}

pub fn read_string<'a>(des: &'a mut SliceDeserializer, section: Section) -> Cow<'a, str> {
    des.execute_at(section.offset as usize, |des| {
        String::from_utf8_lossy(des.read_slice(section.size as usize))
    })
}
