use std::borrow::Cow;

use anyhow::Result;
use common::serde::{Deserializer, Serializer, SliceDeserializer};

mod encoding;
mod preview;

pub(crate) use {encoding::xor_cypher, preview::scale_preview};
pub use {
    encoding::{LayerDecoder, LayerEncoder, LayerEncoderV4, LayerEncoderV5},
    preview::PreviewImage,
};

pub const PAGE_SIZE: u64 = 1 << 32;
pub const DEFAULT_XOR_KEY: u32 = 0x67; // is this from UVTools or did i really pick hex six seven 😭️
pub const DISCLAIMER: &str = "Layout and record format for the ctb and cbddlp file types are the copyrighted programs or codes of CBD Technology (China) Inc..The Customer or User shall not in any manner reproduce, distribute, modify, decompile, disassemble, decrypt, extract, reverse engineer, lease, assign, or sublicense the said programs or codes.";

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

    /// Size, Offset
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
