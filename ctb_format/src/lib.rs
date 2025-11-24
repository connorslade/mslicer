use anyhow::Result;

use common::serde::{Deserializer, Serializer};

mod crypto;
pub mod file;
pub mod layer;
pub mod layer_coding;
pub mod preview;
pub mod resin;

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

    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            offset: des.read_u32_le(),
            size: des.read_u32_le(),
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(self.offset);
        ser.write_u32_le(self.size);
    }

    pub fn deserialize_rev(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            size: des.read_u32_le(),
            offset: des.read_u32_le(),
        })
    }

    pub fn serialize_rev<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(self.size);
        ser.write_u32_le(self.offset);
    }
}

fn read_string(des: &mut Deserializer, section: Section) -> String {
    des.execute_at(section.offset as usize, |des| {
        String::from_utf8_lossy(des.read_bytes(section.size as usize)).into_owned()
    })
}
