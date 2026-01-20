mod oklab;
mod rgb;

pub use self::{
    oklab::OkLab,
    rgb::{LinearRgb, SRgb},
};

/// A good starting color for hue shifting
pub const START_COLOR: OkLab<f32> = OkLab {
    l: 0.65, // lightness
    a: 0.2,  // the magnitude of ⟨a, b⟩ is like the saturation
    b: 0.0,
};
