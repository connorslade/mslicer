use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Goo,
    Svg,
}

impl Format {
    pub const ALL: [Format; 2] = [Format::Goo, Format::Svg];

    pub fn name(&self) -> &'static str {
        match self {
            Format::Goo => "Goo",
            Format::Svg => "Svg",
        }
    }

    pub fn supports_preview(&self) -> bool {
        match self {
            Format::Goo => true,
            Format::Svg => false,
        }
    }
}
