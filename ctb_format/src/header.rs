use anyhow::Result;

use common::serde::Deserializer;

use crate::Section;

#[derive(Debug)]
pub struct Header {
    pub magic: u32,
    pub version: u32,

    pub settings: Section,
    pub signature: Section,
}

impl Header {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            magic: des.read_u32_le(),
            settings: Section::deserialize(des)?,
            version: {
                des.advance_by(4);
                des.read_u32_le()
            },
            signature: Section::deserialize(des)?,
        })
    }
}
