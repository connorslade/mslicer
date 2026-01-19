use anyhow::{Result, bail};

use crate::serde::{Deserializer, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Goo,
    Ctb,
    Svg,
}

impl Format {
    pub const ALL: [Format; 3] = [Format::Goo, Format::Ctb, Format::Svg];

    pub fn from_extension(extension: &str) -> Option<Self> {
        Some(match extension.to_lowercase().as_str() {
            "goo" => Format::Goo,
            "ctb" => Format::Ctb,
            "svg" => Format::Svg,
            _ => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Format::Goo => "Elegoo",
            Format::Ctb => "Chitu Encrypted",
            Format::Svg => "Vector",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Format::Goo => "goo",
            Format::Ctb => "ctb",
            Format::Svg => "svg",
        }
    }

    pub fn supports_preview(&self) -> bool {
        match self {
            Format::Goo => true,
            Format::Ctb => true,
            Format::Svg => false,
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u8(match self {
            Format::Goo => 0,
            Format::Ctb => 1,
            Format::Svg => 2,
        });
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Result<Self> {
        Ok(match des.read_u8() {
            0 => Format::Goo,
            1 => Format::Ctb,
            2 => Format::Svg,
            _ => bail!("Invalid slice format ID"),
        })
    }
}
