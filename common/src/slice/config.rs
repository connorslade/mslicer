use std::borrow::Cow;

use anyhow::Result;
use nalgebra::{Vector2, Vector3};

use crate::{
    misc::lerp,
    serde::{Deserializer, SerdeExt, Serializer},
    slice::format::SliceMode,
    units::{
        CentimetersPerSecond, CubicMilimeters, Milimeter, Milimeters, Minutes, Seconds,
        SquareMilimeters,
    },
};

/// Configuration for slicing a model.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SliceConfig {
    pub mode: SliceMode,
    pub supersample: Supersample,
    pub exposure_remap: ExposureRemap,

    pub platform_resolution: Vector2<u32>,
    pub platform_size: Vector3<Milimeters>,
    pub slice_height: Milimeters,

    pub exposure_config: ExposureConfig,
    pub first_exposure_config: ExposureConfig,
    pub first_layers: Height,
    pub transition_layers: Height,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Supersample {
    pub xy: u8,
    pub z: u8,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExposureRemap {
    pub start: f32,
    pub end: f32,
    pub control: [Vector2<f32>; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Height {
    Layers(u32),
    Distance(Milimeters),
}

/// Layer exposure settings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ExposureConfig {
    pub exposure_time: Seconds,
    pub exposure_delay: Seconds,
    pub pwm: u8,

    pub lift_distance: Milimeters,
    pub lift_speed: CentimetersPerSecond,
    pub retract_speed: CentimetersPerSecond,
}

impl SliceConfig {
    pub fn layer_counts(&self) -> (u32, u32) {
        let slice_height = self.slice_height;
        let first_layers = self.first_layers.layers(slice_height);
        let transition_layers = self.transition_layers.layers(slice_height);

        (first_layers, transition_layers)
    }

    pub fn exposure_config(&self, layer: u32) -> Cow<'_, ExposureConfig> {
        let (first_layers, transition_layers) = self.layer_counts();
        if layer < first_layers {
            Cow::Borrowed(&self.first_exposure_config)
        } else if layer < first_layers + transition_layers {
            let t = (layer - first_layers) as f32 / transition_layers as f32;
            Cow::Owned(self.first_exposure_config.lerp(&self.exposure_config, t))
        } else {
            Cow::Borrowed(&self.exposure_config)
        }
    }

    pub fn default_height(&self, layer: u32) -> Milimeters {
        self.slice_height * (layer + 1) as f32
    }

    pub fn pixel_size(&self) -> Vector2<Milimeters> {
        Vector2::new(
            self.platform_size.x / self.platform_resolution.x as f32,
            self.platform_size.y / self.platform_resolution.y as f32,
        )
    }

    pub fn pixel_area(&self) -> SquareMilimeters {
        let size = self.pixel_size();
        size.x * size.y
    }

    pub fn voxel_volume(&self) -> CubicMilimeters {
        self.pixel_area() * self.slice_height
    }

    pub fn mm_to_px(&self, mm: Vector2<f32>) -> Vector2<f32> {
        mm.component_mul(&self.platform_resolution.cast())
            .component_div(&self.platform_size.xy().map(|x| x.get::<Milimeter>()))
    }

    pub fn print_time(&self, layers: u32) -> Seconds {
        let exp = &self.exposure_config;
        let fexp = &self.first_exposure_config;

        let slice_height = self.slice_height;
        let first_layers = self.first_layers.layers(slice_height).min(layers);
        let transition_layers = layers
            .saturating_sub(first_layers)
            .min(self.transition_layers.layers(slice_height));
        let regular_layers = layers
            .saturating_sub(first_layers)
            .saturating_sub(transition_layers);

        let layer_time = exp.print_time();
        let bottom_layer_time = fexp.print_time();

        regular_layers as f32 * layer_time
            + first_layers as f32 * bottom_layer_time
            + transition_layers as f32 * (bottom_layer_time + layer_time) / 2.0
    }
}

impl ExposureRemap {
    pub fn points(&self) -> [Vector2<f32>; 4] {
        let (start, end) = (Vector2::new(0.0, self.start), Vector2::new(1.0, self.end));
        [start, start + self.control[0], end + self.control[1], end]
    }

    pub fn bezier(&self, t: f32) -> Vector2<f32> {
        let [p1, p2, p3, p4] = self.points();
        let a = lerp(p2, p3, t);
        let b = lerp(lerp(p1, p2, t), a, t);
        let c = lerp(a, lerp(p3, p4, t), t);
        lerp(b, c, t)
    }

    pub fn remap(&self, x: f32) -> f32 {
        let (mut low, mut high) = (0.0, 1.0);
        for _ in 0..20 {
            let mid = (low + high) / 2.0;
            if self.bezier(mid).x < x {
                low = mid;
            } else {
                high = mid;
            }
        }

        self.bezier((low + high) * 0.5).y
    }

    pub fn table(&self) -> [u8; 256] {
        let mut out = [0; 256];
        for (i, x) in out.iter_mut().enumerate() {
            *x = (self.remap(i as f32 / 255.0) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
        out
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_be(self.start);
        ser.write_f32_be(self.end);
        self.control[0].serialize(ser);
        self.control[1].serialize(ser);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Result<Self> {
        Ok(Self {
            start: des.read_f32_be(),
            end: des.read_f32_be(),
            control: [Vector2::deserialize(des), Vector2::deserialize(des)],
        })
    }
}

impl Height {
    pub fn type_name(&self) -> &str {
        match self {
            Height::Layers(_) => "Layers",
            Height::Distance(_) => "Distance",
        }
    }

    pub fn layers(&self, slice_height: Milimeters) -> u32 {
        match self {
            Height::Layers(x) => *x,
            Height::Distance(length) => (*length / slice_height).round() as u32,
        }
    }

    pub fn distance(&self, slice_height: Milimeters) -> Milimeters {
        match self {
            Height::Layers(x) => *x as f32 * slice_height,
            Height::Distance(length) => *length,
        }
    }

    pub fn flip(&self, slice_height: Milimeters) -> Self {
        match self {
            Height::Layers(x) => Height::Distance(*x as f32 * slice_height),
            Height::Distance(length) => Height::Layers((*length / slice_height).round() as u32),
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        match self {
            Height::Layers(layers) => {
                ser.write_u8(0);
                ser.write_u32_be(*layers);
            }
            Height::Distance(length) => {
                ser.write_u8(1);
                ser.write_f32_be(length.get::<Milimeter>());
            }
        }
    }

    pub fn deserialize<T: Deserializer>(des: &mut T, version: u16) -> Self {
        if version < 15 {
            return Self::Layers(des.read_u32_be());
        }

        let tag = des.read_u8();
        match tag {
            0 => Self::Layers(des.read_u32_be()),
            1 => Self::Distance(Milimeters::new(des.read_f32_be())),
            _ => Self::Layers(0), // unreachable
        }
    }
}

impl SliceConfig {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.mode.serialize(ser);
        self.supersample.serialize(ser);
        self.exposure_remap.serialize(ser);
        self.platform_resolution.serialize(ser);
        self.platform_size.map(|x| x.raw()).serialize(ser);
        ser.write_f32_be(self.slice_height.raw());
        self.exposure_config.serialize(ser);
        self.first_exposure_config.serialize(ser);
        self.first_layers.serialize(ser);
        self.transition_layers.serialize(ser);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T, version: u16) -> Result<Self> {
        Ok(Self {
            mode: if version < 6 {
                [SliceMode::Raster, SliceMode::Vector][(des.read_u8() == 2) as usize]
            } else {
                SliceMode::deserialize(des)?
            },
            supersample: match version {
                ..5 => Default::default(),
                5..14 => Supersample::splat(des.read_u8()),
                _ => Supersample::deserialize(des),
            },
            exposure_remap: if version < 8 {
                Default::default()
            } else {
                ExposureRemap::deserialize(des)?
            },
            platform_resolution: Vector2::deserialize(des),
            platform_size: Vector3::deserialize(des).map(Milimeters::new),
            slice_height: Milimeters::new(des.read_f32_be()),
            exposure_config: ExposureConfig::deserialize(des, version),
            first_exposure_config: ExposureConfig::deserialize(des, version),
            first_layers: Height::deserialize(des, version),
            transition_layers: Height::deserialize(des, version),
        })
    }
}

impl ExposureConfig {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_be(self.exposure_time.raw());
        ser.write_f32_be(self.exposure_delay.raw());
        ser.write_u8(self.pwm);
        ser.write_f32_be(self.lift_distance.raw());
        ser.write_f32_be(self.lift_speed.raw());
        ser.write_f32_be(self.retract_speed.raw());
    }

    pub fn deserialize<T: Deserializer>(des: &mut T, version: u16) -> Self {
        Self {
            exposure_time: Seconds::new(des.read_f32_be()),
            exposure_delay: Seconds::new(if version < 7 { 0.0 } else { des.read_f32_be() }),
            pwm: if version < 3 { 255 } else { des.read_u8() },

            lift_distance: Milimeters::new(des.read_f32_be()),
            lift_speed: CentimetersPerSecond::new(des.read_f32_be()),
            retract_speed: {
                (version < 13).then(|| des.advance_by(4));
                CentimetersPerSecond::new(des.read_f32_be())
            },
        }
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            exposure_time: lerp(self.exposure_time, other.exposure_time, t),
            exposure_delay: lerp(self.exposure_delay, other.exposure_delay, t),
            pwm: lerp(self.pwm as f32, other.pwm as f32, t) as u8,

            lift_distance: lerp(self.lift_distance, other.lift_distance, t),
            lift_speed: lerp(self.lift_speed, other.lift_speed, t),
            retract_speed: lerp(self.retract_speed, other.retract_speed, t),
        }
    }

    pub fn print_time(&self) -> Seconds {
        self.exposure_time
            + self.lift_distance / self.lift_speed
            + self.lift_distance / self.retract_speed
            + self.exposure_delay
    }
}

impl Supersample {
    pub fn splat(value: u8) -> Self {
        Self {
            xy: value,
            z: value,
        }
    }

    pub fn is_anisotropic(&self) -> bool {
        self.xy != self.z
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u8(self.xy);
        ser.write_u8(self.z);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            xy: des.read_u8(),
            z: des.read_u8(),
        }
    }
}

impl Default for SliceConfig {
    fn default() -> Self {
        Self {
            mode: SliceMode::Raster,
            supersample: Default::default(),
            exposure_remap: Default::default(),

            platform_resolution: Vector2::new(11_520, 5_120),
            platform_size: Vector3::new(218.88, 122.904, 260.0).map(Milimeters::new),
            slice_height: Milimeters::new(0.05),
            exposure_config: ExposureConfig {
                exposure_time: Seconds::new(3.0),
                ..Default::default()
            },
            first_exposure_config: ExposureConfig {
                exposure_time: Seconds::new(30.0),
                ..Default::default()
            },
            first_layers: Height::Distance(Milimeters::new(0.5)),
            transition_layers: Height::Distance(Milimeters::new(0.2)),
        }
    }
}

impl Default for Supersample {
    fn default() -> Self {
        Self { xy: 1, z: 1 }
    }
}

impl Default for ExposureRemap {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 1.0,
            control: [Vector2::new(0.25, 0.25), Vector2::new(-0.25, -0.25)],
        }
    }
}

impl Default for ExposureConfig {
    fn default() -> Self {
        Self {
            exposure_delay: Seconds::new(1.0),
            exposure_time: Seconds::new(3.0),
            pwm: 255,

            lift_distance: Milimeters::new(5.0),
            lift_speed: (Milimeters::new(330.0) / Minutes::new(1.0)).convert(),
            retract_speed: (Milimeters::new(330.0) / Minutes::new(1.0)).convert(),
        }
    }
}
