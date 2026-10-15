use std::f32::consts::PI;

use common::color::{LinearRgb, OkLab, SRgb};

pub fn invert_color(linear: LinearRgb<f32>) -> SRgb<f32> {
    OkLab::from_linear_srgb(linear)
        .hue_shift(PI)
        .to_linear_srgb()
        .to_srgb()
}
