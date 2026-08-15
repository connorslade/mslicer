use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    pub max_buffer_size: u64,
    pub style: RenderStyle,
    pub projection: Projection,
    pub grid_size: f32,
    pub normals: bool,
    pub overhangs: (bool, f32),
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RenderStyle {
    Normals,
    RandomTriangle,
    Rendered,
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Projection {
    Perspective,
    Orthographic,
}

impl RenderStyle {
    pub const ALL: [Self; 3] = [Self::Normals, Self::RandomTriangle, Self::Rendered];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Normals => "Normals",
            Self::RandomTriangle => "Triangles",
            Self::Rendered => "Rendered",
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            grid_size: 12.16,
            overhangs: (false, 30.0),
            style: RenderStyle::Rendered,
            projection: Projection::Perspective,
            max_buffer_size: 512 << 20,
            normals: false,
        }
    }
}
