use anyhow::Result;
use nalgebra::{Vector2, Vector3};

use crate::{
    serde::{Deserializer, SerdeExt, Serializer},
    slice::format::SliceMode,
    units::{
        CentimetersPerSecond, CubicMilimeters, Milimeter, Milimeters, Minutes, Seconds,
        SquareMilimeters,
    },
};

/// Configuration for slicing a model.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SliceConfig {
    pub mode: SliceMode,
    pub supersample: u8,

    pub platform_resolution: Vector2<u32>,
    pub platform_size: Vector3<Milimeters>,
    pub slice_height: Milimeters,

    pub exposure_config: ExposureConfig,
    pub first_exposure_config: ExposureConfig,
    pub first_layers: u32,
    pub transition_layers: u32,
}

/// Layer exposure settings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ExposureConfig {
    pub exposure_time: Seconds,
    pub pwm: u8,

    pub lift_distance: Milimeters,
    pub lift_speed: CentimetersPerSecond,

    pub retract_distance: Milimeters,
    pub retract_speed: CentimetersPerSecond,
}

impl SliceConfig {
    pub fn exposure_config(&self, layer: u32) -> &ExposureConfig {
        if layer < self.first_layers {
            &self.first_exposure_config
        } else {
            &self.exposure_config
        }
    }

    pub fn pixel_area(&self) -> SquareMilimeters {
        let x = self.platform_size.x / self.platform_resolution.x as f32;
        let y = self.platform_size.y / self.platform_resolution.y as f32;
        x * y
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

        let first_layers = self.first_layers.min(layers);
        let transition_layers = layers
            .saturating_sub(self.first_layers)
            .min(self.transition_layers);
        let regular_layers = layers.saturating_sub(transition_layers);

        let layer_time = exp.exposure_time
            + exp.lift_distance / exp.lift_speed
            + exp.retract_distance / exp.retract_speed;
        let bottom_layer_time = fexp.exposure_time
            + fexp.lift_distance / fexp.lift_speed
            + fexp.retract_distance / fexp.retract_speed;

        regular_layers as f32 * layer_time
            + first_layers as f32 * bottom_layer_time
            + transition_layers as f32 * (bottom_layer_time + layer_time) / 2.0
    }
}

impl Default for SliceConfig {
    fn default() -> Self {
        Self {
            mode: SliceMode::Raster,
            supersample: 0,

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
            first_layers: 3,
            transition_layers: 10,
        }
    }
}

impl Default for ExposureConfig {
    fn default() -> Self {
        Self {
            exposure_time: Seconds::new(3.0),
            pwm: 255,

            lift_distance: Milimeters::new(5.0),
            lift_speed: (Milimeters::new(330.0) / Minutes::new(1.0)).convert(),

            retract_distance: Milimeters::new(5.0),
            retract_speed: (Milimeters::new(330.0) / Minutes::new(1.0)).convert(),
        }
    }
}

impl SliceConfig {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.mode.serialize(ser);
        ser.write_u8(self.supersample);
        self.platform_resolution.serialize(ser);
        self.platform_size.map(|x| x.raw()).serialize(ser);
        ser.write_f32_be(self.slice_height.raw());
        self.exposure_config.serialize(ser);
        self.first_exposure_config.serialize(ser);
        ser.write_u32_be(self.first_layers);
        ser.write_u32_be(self.transition_layers);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T, version: u16) -> Result<Self> {
        Ok(Self {
            mode: if version < 6 {
                [SliceMode::Raster, SliceMode::Vector][(des.read_u8() == 2) as usize]
            } else {
                SliceMode::deserialize(des)?
            },
            supersample: if version < 5 { 1 } else { des.read_u8() },
            platform_resolution: Vector2::deserialize(des),
            platform_size: Vector3::deserialize(des).map(Milimeters::new),
            slice_height: Milimeters::new(des.read_f32_be()),
            exposure_config: ExposureConfig::deserialize(des, version),
            first_exposure_config: ExposureConfig::deserialize(des, version),
            first_layers: des.read_u32_be(),
            transition_layers: des.read_u32_be(),
        })
    }
}

impl ExposureConfig {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_be(self.exposure_time.raw());
        ser.write_u8(self.pwm);
        ser.write_f32_be(self.lift_distance.raw());
        ser.write_f32_be(self.lift_speed.raw());
        ser.write_f32_be(self.retract_distance.raw());
        ser.write_f32_be(self.retract_speed.raw());
    }

    pub fn deserialize<T: Deserializer>(des: &mut T, version: u16) -> Self {
        Self {
            exposure_time: Seconds::new(des.read_f32_be()),
            pwm: if version < 3 { 255 } else { des.read_u8() },

            lift_distance: Milimeters::new(des.read_f32_be()),
            lift_speed: CentimetersPerSecond::new(des.read_f32_be()),

            retract_distance: Milimeters::new(des.read_f32_be()),
            retract_speed: CentimetersPerSecond::new(des.read_f32_be()),
        }
    }
}
