use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SlicedConfig {
    pub coordinate_space: SlicePreviewCoordinateSpace,
    pub view: SlicePreviewView,
    pub multisample: u32,
    pub sidebar: bool,
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

impl SlicePreviewCoordinateSpace {
    pub const ALL: &[Self] = &[Self::ScreenSpace, Self::WorldSpace];

    pub fn name(&self) -> &str {
        match self {
            Self::ScreenSpace => "Screen Space",
            Self::WorldSpace => "World Space",
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

impl Default for SlicedConfig {
    fn default() -> Self {
        Self {
            coordinate_space: SlicePreviewCoordinateSpace::WorldSpace,
            view: SlicePreviewView::BuildPlate,
            multisample: 8,
            sidebar: true,
        }
    }
}
