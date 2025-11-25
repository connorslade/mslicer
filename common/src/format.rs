use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Goo,
    Ctb,
    Svg,
}

impl Format {
    pub const ALL: [Format; 3] = [Format::Goo, Format::Ctb, Format::Svg];

    pub fn from_extention(extention: &str) -> Option<Self> {
        Some(match extention.to_lowercase().as_str() {
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

    pub fn extention(&self) -> &'static str {
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
}
