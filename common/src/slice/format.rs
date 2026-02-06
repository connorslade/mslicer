use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::serde::{Deserializer, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    Goo,
    Ctb,
    NanoDLP,
    Svg,
}

impl Format {
    pub const ALL: [Format; 4] = [Format::Goo, Format::Ctb, Format::NanoDLP, Format::Svg];

    pub fn from_extension(extension: &str) -> Option<Self> {
        Some(match extension.to_lowercase().as_str() {
            "goo" => Format::Goo,
            "ctb" => Format::Ctb,
            "nanodlp" => Format::NanoDLP,
            "svg" => Format::Svg,
            _ => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Format::Goo => "Elegoo",
            Format::Ctb => "Chitu Encrypted",
            Format::NanoDLP => "NanoDLP",
            Format::Svg => "Vector",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Format::Goo => "goo",
            Format::Ctb => "ctb",
            Format::NanoDLP => "nanodlp",
            Format::Svg => "svg",
        }
    }

    pub fn supports_preview(&self) -> bool {
        matches!(self, Format::Goo | Format::Ctb | Format::NanoDLP)
    }

    pub fn supports_remote_send(&self) -> bool {
        matches!(self, Format::Goo | Format::Ctb)
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u8(match self {
            Format::Goo => 0,
            Format::Ctb => 1,
            Format::NanoDLP => 3,
            Format::Svg => 2,
        });
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Result<Self> {
        Ok(match des.read_u8() {
            0 => Format::Goo,
            1 => Format::Ctb,
            3 => Format::NanoDLP,
            2 => Format::Svg,
            _ => bail!("Invalid slice format ID"),
        })
    }
}
