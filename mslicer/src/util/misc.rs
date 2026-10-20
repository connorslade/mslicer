use std::{
    f32::consts::PI,
    ops::{Deref, DerefMut},
};

use common::color::{LinearRgb, OkLab, SRgb};

pub fn invert_color(linear: LinearRgb<f32>) -> SRgb<f32> {
    OkLab::from_linear_srgb(linear)
        .hue_shift(PI)
        .to_linear_srgb()
        .to_srgb()
}

pub struct DefaultNum<const V: u64, T> {
    inner: T,
}

impl<const V: u64> Default for DefaultNum<V, u8> {
    fn default() -> Self {
        Self { inner: V as _ }
    }
}

impl<const V: u64, T> DerefMut for DefaultNum<V, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<const V: u64, T> Deref for DefaultNum<V, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
