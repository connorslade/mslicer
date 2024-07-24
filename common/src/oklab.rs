use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct OkLab<T> {
    pub l: T,
    pub a: T,
    pub b: T,
}

#[derive(Debug, Clone, Copy)]
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

    pub fn to_srgb(&self) -> Rgb<f32> {
        oklab_to_linear_srgb(*self)
    }

    pub fn from_srgb(c: Rgb<f32>) -> Self {
        linear_srgb_to_oklab(c)
    }

    pub fn to_lrgb(&self) -> Rgb<u8> {
        let srgb = self.to_srgb();
        Rgb {
            r: (to_gamma(srgb.r) * 255.0).round() as u8,
            g: (to_gamma(srgb.g) * 255.0).round() as u8,
            b: (to_gamma(srgb.b) * 255.0).round() as u8,
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

impl<T> Rgb<T> {
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> Rgb<U> {
        Rgb {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }
}

pub fn linear_srgb_to_oklab(c: Rgb<f32>) -> OkLab<f32> {
    let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    OkLab {
        l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    }
}

pub fn oklab_to_linear_srgb(c: OkLab<f32>) -> Rgb<f32> {
    let l_ = c.l + 0.3963377774 * c.a + 0.2158037573 * c.b;
    let m_ = c.l - 0.1055613458 * c.a - 0.0638541728 * c.b;
    let s_ = c.l - 0.0894841775 * c.a - 1.2914855480 * c.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    Rgb {
        r: 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        g: -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        b: -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    }
}

fn to_gamma(u: f32) -> f32 {
    if u >= 0.0031308 {
        (1.055) * u.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * u
    }
}
