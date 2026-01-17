mod oklab;
mod rgb;

pub use self::{
    oklab::OkLab,
    rgb::{LinearRgb, SRgb},
};

/// A good starting color for hue shifting
pub const START_COLOR: OkLab<f32> = OkLab {
    l: 0.65,
    a: 0.178,
    b: -0.116,
};
