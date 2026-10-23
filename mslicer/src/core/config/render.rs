use common::{color::SRgb, units::Seconds};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::render::camera::Camera;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    // system
    pub max_buffer_size: u64,
    pub stats_average_time: Seconds,

    // basic rendering
    pub style: RenderStyle,
    pub projection: Projection,
    pub ambient_occlusion: AmbientOcclusion,
    pub anti_aliasing: AntiAliasing,

    pub preview: Preview,

    // extras
    pub grid_size: f32,
    pub normals: DebugRender,
    pub bounding_box: DebugRender,
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
    pub mode: AntiAliasMode,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntiAliasMode {
    #[default]
    Fxaa,
    Taa,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preview {
    pub size: Vector2<u32>,
    pub background: SRgb<f32>,

    pub projection: Projection,
    pub fov: f32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugRender {
    pub enabled: bool,
    pub color: SRgb<f32>,
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

impl AntiAliasMode {
    pub const ALL: [Self; 1] = [Self::Fxaa]; //, Self::Taa];

    pub fn name(&self) -> &str {
        match self {
            AntiAliasMode::Fxaa => "FXAA",
            AntiAliasMode::Taa => "TAA",
        }
    }
}

impl AntiAliasing {
    pub fn fxaa(&self) -> bool {
        self.enabled && self.mode == AntiAliasMode::Fxaa
    }

    pub fn taa(&self) -> bool {
        self.enabled && self.mode == AntiAliasMode::Taa
    }
}

impl DebugRender {
    pub fn new() -> Self {
        Self {
            enabled: false,
            color: SRgb::splat(f32::MAX),
        }
    }

    pub fn with_color(self, color: SRgb<f32>) -> Self {
        Self { color, ..self }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 512 << 20,
            stats_average_time: Seconds::new(1.0),

            style: RenderStyle::Rendered,
            projection: Projection::Perspective,
            ambient_occlusion: Default::default(),
            anti_aliasing: Default::default(),

            preview: Preview::default(),

            grid_size: 12.16,
            overhangs: (false, 30.0),
            normals: DebugRender::new().with_color(SRgb::new(0.5, 0.5, 1.0)),
            bounding_box: DebugRender::new().with_color(SRgb::new(1.0, 0.5, 1.0)),
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
        Self {
            enabled: true,
            mode: Default::default(),
        }
    }
}

impl Default for Preview {
    fn default() -> Self {
        Self {
            size: Vector2::new(400, 300),
            background: SRgb::splat(0.0),

            projection: Projection::Orthographic,
            fov: Camera::default().fov,
        }
    }
}
