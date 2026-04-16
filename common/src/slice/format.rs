use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::serde::{Deserializer, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SliceMode {
    Raster,
    Vector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RasterFormat {
    Goo,
    Ctb,
    NanoDLP,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorFormat {
    Svg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    Raster(RasterFormat),
    Vector(VectorFormat),
}

impl SliceMode {
    pub const ALL: [Self; 2] = [Self::Raster, Self::Vector];

    pub fn name(&self) -> &str {
        match self {
            SliceMode::Raster => "Raster",
            SliceMode::Vector => "Vector",
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u8(match self {
            SliceMode::Raster => 0,
            SliceMode::Vector => 1,
        });
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Result<Self> {
        Ok(match des.read_u8() {
            0 => SliceMode::Raster,
            1 => SliceMode::Vector,
            _ => bail!("Invalid slice format type"),
        })
    }
}

impl RasterFormat {
    pub const ALL: [Self; 3] = [Self::Goo, Self::Ctb, Self::NanoDLP];

    pub fn name(&self) -> &str {
        match self {
            Self::Goo => "Elegoo",
            Self::Ctb => "Chitu Encrypted",
            Self::NanoDLP => "NanoDLP",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Goo => "goo",
            Self::Ctb => "ctb",
            Self::NanoDLP => "nanodlp",
        }
    }

    pub fn from_extension(extension: &str) -> Option<Self> {
        Some(match extension.to_lowercase().as_str() {
            "goo" => Self::Goo,
            "ctb" => Self::Ctb,
            "nanodlp" => Self::NanoDLP,
            _ => return None,
        })
    }
}

impl VectorFormat {
    pub const ALL: [Self; 1] = [Self::Svg];

    pub fn name(&self) -> &str {
        match self {
            VectorFormat::Svg => "Svg",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            VectorFormat::Svg => "svg",
        }
    }
}

impl Format {
    pub const VECTOR: [Format; 1] = [Format::Vector(VectorFormat::Svg)];
    pub const RASTER: [Format; 3] = [
        Format::Raster(RasterFormat::Ctb),
        Format::Raster(RasterFormat::Goo),
        Format::Raster(RasterFormat::NanoDLP),
    ];

    pub fn extension(&self) -> &str {
        match self {
            Format::Raster(format) => format.extension(),
            Format::Vector(format) => format.extension(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Format::Raster(format) => format!("{} (.{})", format.name(), format.extension()),
            Format::Vector(format) => format!("{} (.{})", format.name(), format.extension()),
        }
    }

    pub fn as_raster(&self) -> Option<RasterFormat> {
        match self {
            Format::Raster(format) => Some(*format),
            _ => None,
        }
    }

    pub fn as_vector(&self) -> Option<VectorFormat> {
        match self {
            Format::Vector(format) => Some(*format),
            _ => None,
        }
    }
}

impl From<RasterFormat> for Format {
    fn from(value: RasterFormat) -> Self {
        Self::Raster(value)
    }
}

impl From<VectorFormat> for Format {
    fn from(value: VectorFormat) -> Self {
        Self::Vector(value)
    }
}
