// Reference: https://www.w3.org/Graphics/Color/srgb

use std::mem;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearRgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SRgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

impl<T> LinearRgb<T> {
    pub fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T; 3] {
        unsafe { mem::transmute(self) }
    }

    pub fn as_slice(&self) -> &[T; 3] {
        unsafe { mem::transmute(self) }
    }
}

impl<T> SRgb<T> {
    pub fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T; 3] {
        unsafe { mem::transmute(self) }
    }

    pub fn as_slice(&self) -> &[T; 3] {
        unsafe { mem::transmute(self) }
    }
}

impl LinearRgb<f32> {
    pub fn to_srgb(&self) -> SRgb<f32> {
        fn convert(v: f32) -> f32 {
            if v <= 0.0031308 {
                12.92 * v
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            }
        }

        SRgb {
            r: convert(self.r),
            g: convert(self.g),
            b: convert(self.b),
        }
    }
}

impl SRgb<f32> {
    pub fn to_linear_rgb(&self) -> LinearRgb<f32> {
        fn convert(v: f32) -> f32 {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }

        LinearRgb {
            r: convert(self.r),
            g: convert(self.g),
            b: convert(self.b),
        }
    }
}

impl<T: Copy> LinearRgb<T> {
    pub fn repeat(v: T) -> Self {
        Self::new(v, v, v)
    }
}

impl<T: Copy> SRgb<T> {
    pub fn repeat(v: T) -> Self {
        Self::new(v, v, v)
    }
}

impl<T> From<LinearRgb<T>> for Vector3<T> {
    fn from(value: LinearRgb<T>) -> Self {
        Vector3::new(value.r, value.g, value.b)
    }
}

impl<T> From<SRgb<T>> for Vector3<T> {
    fn from(value: SRgb<T>) -> Self {
        Vector3::new(value.r, value.g, value.b)
    }
}

impl<T: Copy> From<Vector3<T>> for LinearRgb<T> {
    fn from(value: Vector3<T>) -> Self {
        let [r, g, b] = value.data.0[0];
        Self { r, g, b }
    }
}

impl<T: Copy> From<Vector3<T>> for SRgb<T> {
    fn from(value: Vector3<T>) -> Self {
        let [r, g, b] = value.data.0[0];
        Self { r, g, b }
    }
}
