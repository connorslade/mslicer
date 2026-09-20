use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SlicedConfig {
    pub coordinate_space: SlicePreviewCoordinateSpace,
    pub view: SlicePreviewView,
    pub multisample: u32,
    pub sidebar: bool,

    pub resin_density: f32, // g/mL
    pub resin_cost: f32,    // ¤/mL
    pub currency: Currency,
}

#[derive(Default, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum SlicePreviewCoordinateSpace {
    ScreenSpace,
    #[default]
    WorldSpace,
}

#[derive(Default, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum SlicePreviewView {
    Screen,
    #[default]
    BuildPlate,
}

#[derive(Default, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum Currency {
    Euro,
    #[default]
    Dollar,
    Yen,
    Generic,
}

impl SlicePreviewCoordinateSpace {
    pub const ALL: &[Self] = &[Self::ScreenSpace, Self::WorldSpace];

    pub fn name(&self) -> &str {
        match self {
            Self::ScreenSpace => "Screen",
            Self::WorldSpace => "World",
        }
    }
}

impl SlicePreviewView {
    pub const ALL: &[Self] = &[Self::Screen, Self::BuildPlate];

    pub fn name(&self) -> &str {
        match self {
            Self::Screen => "Screen",
            Self::BuildPlate => "Build Plate",
        }
    }
}

impl Currency {
    pub const ALL: [Self; 4] = [Self::Euro, Self::Dollar, Self::Yen, Self::Generic];

    pub fn name(&self) -> &str {
        match self {
            Currency::Euro => "Euro",
            Currency::Dollar => "Dollar",
            Currency::Yen => "Yen",
            Currency::Generic => "Generic",
        }
    }

    pub fn symbol(&self) -> char {
        match self {
            Currency::Euro => '€',
            Currency::Dollar => '$',
            Currency::Yen => '¥',
            Currency::Generic => '¤',
        }
    }
}

impl Default for SlicedConfig {
    fn default() -> Self {
        Self {
            coordinate_space: Default::default(),
            view: Default::default(),
            multisample: 8,
            sidebar: true,

            resin_density: 1.1,
            resin_cost: 19.79,
            currency: Currency::default(),
        }
    }
}
