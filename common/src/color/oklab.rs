#![allow(clippy::excessive_precision)]

use std::f32::consts::PI;

use crate::color::rgb::LinearRgb;

#[derive(Debug, Clone, Copy)]
pub struct OkLab<T> {
    pub l: T,
    pub a: T,
    pub b: T,
}

impl OkLab<f32> {
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        OkLab { l, a, b }
    }

    #[rustfmt::skip]
    pub fn to_linear_srgb(&self) -> LinearRgb<f32> {
        let l = self.l + 0.396_337_780 * self.a + 0.215_803_76 * self.b;
        let m = self.l - 0.105_561_346 * self.a - 0.063_854_17 * self.b;
        let s = self.l - 0.089_484_180 * self.a - 1.291_485_50 * self.b;

        let l = l * l * l;
        let m = m * m * m;
        let s = s * s * s;

        LinearRgb {
            r:  4.076_741_700_0 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            g: -1.268_438_000_0 * l + 2.609_757_4 * m - 0.341_319_38 * s,
            b: -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_70 * s,
        }
    }

    #[rustfmt::skip]
    pub fn from_linear_srgb(c: LinearRgb<f32>) -> Self {
        let l = 0.412_221_46 * c.r + 0.536_332_55 * c.g + 0.051_445_995 * c.b;
        let m = 0.211_903_50 * c.r + 0.680_699_50 * c.g + 0.107_396_960 * c.b;
        let s = 0.088_302_46 * c.r + 0.281_718_85 * c.g + 0.629_978_700 * c.b;

        let l = l.cbrt();
        let m = m.cbrt();
        let s = s.cbrt();

        OkLab {
            l: 0.210_454_260 * l + 0.793_617_80 * m - 0.004_072_047 * s,
            a: 1.977_998_500 * l - 2.428_592_20 * m + 0.450_593_700 * s,
            b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_770 * s,
        }
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
