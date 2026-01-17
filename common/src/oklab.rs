use std::{f32::consts::PI, mem};

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct OkLab<T> {
    pub l: T,
    pub a: T,
    pub b: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Rgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

/// A good starting color for hue shifting
pub const START_COLOR: OkLab<f32> = OkLab {
    l: 0.773,
    a: 0.1131,
    b: 0.0,
};

impl OkLab<f32> {
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        OkLab { l, a, b }
    }

    pub fn to_linear_srgb(&self) -> Rgb<f32> {
        oklab_to_linear_srgb(*self)
    }

    pub fn from_linear_srgb(c: Rgb<f32>) -> Self {
        linear_srgb_to_oklab(c)
    }

    pub fn to_srgb(&self) -> Rgb<f32> {
        self.to_linear_srgb().map(|x| x.powf(2.2))
    }

    pub fn hue_shift(&self, shift: f32) -> Self {
        let hue = self.b.atan2(self.a);
        let chroma = (self.a * self.a + self.b * self.b).sqrt();

        let hue = (hue + shift) % (2.0 * PI);

        let a = chroma * hue.cos();
        let b = chroma * hue.sin();

        Self { l: self.l, a, b }
    }
}

impl<T> Rgb<T> {
    pub fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T; 3] {
        unsafe { mem::transmute(self) }
    }

    pub fn as_vector(self) -> Vector3<T> {
        Vector3::new(self.r, self.g, self.b)
    }

    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> Rgb<U> {
        Rgb {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }
}

impl<T: Copy> Rgb<T> {
    pub fn repeat(v: T) -> Self {
        Self::new(v, v, v)
    }
}

pub fn linear_srgb_to_oklab(c: Rgb<f32>) -> OkLab<f32> {
    let l = 0.412_221_46 * c.r + 0.536_332_55 * c.g + 0.051_445_995 * c.b;
    let m = 0.211_903_5 * c.r + 0.680_699_5 * c.g + 0.107_396_96 * c.b;
    let s = 0.088_302_46 * c.r + 0.281_718_85 * c.g + 0.629_978_7 * c.b;

    let l = l.cbrt();
    let m = m.cbrt();
    let s = s.cbrt();

    OkLab {
        l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    }
}

pub fn oklab_to_linear_srgb(c: OkLab<f32>) -> Rgb<f32> {
    let l = c.l + 0.396_337_78 * c.a + 0.215_803_76 * c.b;
    let m = c.l - 0.105_561_346 * c.a - 0.063_854_17 * c.b;
    let s = c.l - 0.089_484_18 * c.a - 1.291_485_5 * c.b;

    let l = l * l * l;
    let m = m * m * m;
    let s = s * s * s;

    Rgb {
        r: 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        g: -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        b: -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    }
}

fn to_gamma(u: f32) -> f32 {
    if u >= 0.0031308 {
        (1.055) * u.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * u
    }
}
