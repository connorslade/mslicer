use anyhow::Result;

use common::serde::Deserializer;

mod crypto;
pub mod file;
pub mod layer;
pub mod layer_coding;
pub mod preview;
pub mod resin;
pub mod settings;

#[derive(Debug)]
pub struct Section {
    pub size: u32,
    pub offset: u32,
}

impl Section {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            offset: des.read_u32_le(),
            size: des.read_u32_le(),
        })
    }

    pub fn deserialize_rev(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            size: des.read_u32_le(),
            offset: des.read_u32_le(),
        })
    }
}

fn read_string(des: &mut Deserializer, section: Section) -> String {
    des.execute_at(section.offset as usize, |des| {
        String::from_utf8_lossy(des.read_bytes(section.size as usize)).into_owned()
    })
}
