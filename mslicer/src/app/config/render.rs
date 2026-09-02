use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    // system
    pub max_buffer_size: u64,

    // basic rendering
    pub style: RenderStyle,
    pub projection: Projection,
    pub ambient_occlusion: AmbientOcclusion,
    pub anti_aliasing: AntiAliasing,

    // extras
    pub grid_size: f32,
    pub normals: bool,
    pub overhangs: (bool, f32),
    pub basis_size: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AmbientOcclusion {
    pub enabled: bool,
    pub scale: f32,
    pub samples: u32,
    pub range: f32,
    pub bias: f32,

    pub blur_radius: u32,
    pub blur_spatial: f32,
    pub blur_depth: f32,
    pub blur_normal: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AntiAliasing {
    pub enabled: bool,
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
            max_buffer_size: 512 << 20,

            style: RenderStyle::Rendered,
            projection: Projection::Perspective,
            ambient_occlusion: Default::default(),
            anti_aliasing: Default::default(),

            grid_size: 12.16,
            overhangs: (false, 30.0),
            normals: false,
            basis_size: 100.0,
        }
    }
}

impl Default for AmbientOcclusion {
    fn default() -> Self {
        Self {
            enabled: false,
            scale: 1.0,
            samples: 50,
            range: 0.5,
            bias: 0.0,

            blur_radius: 3,
            blur_spatial: 1.5,
            blur_depth: 0.1,
            blur_normal: 0.1,
        }
    }
}

impl Default for AntiAliasing {
    fn default() -> Self {
        Self { enabled: true }
    }
}
