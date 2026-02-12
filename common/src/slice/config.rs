use anyhow::Result;
use nalgebra::{Vector2, Vector3};

use crate::{
    serde::{Deserializer, SerdeExt, Serializer},
    slice::Format,
    units::{
        CentimetersPerSecond, CubicMilimeters, Milimeters, Minutes, Seconds, SquareMilimeters,
    },
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SliceConfig {
    pub format: Format,

    pub platform_resolution: Vector2<u32>,
    pub platform_size: Vector3<Milimeters>,
    pub slice_height: Milimeters,

    pub exposure_config: ExposureConfig,
    pub first_exposure_config: ExposureConfig,
    pub first_layers: u32,
    pub transition_layers: u32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExposureConfig {
    pub exposure_time: Seconds,
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

    pub fn print_time(&self, layers: u32) -> Seconds {
        let exp = &self.exposure_config;
        let fexp = &self.first_exposure_config;

        let layer_time = exp.exposure_time
            + exp.lift_distance / exp.lift_speed
            + exp.retract_distance / exp.retract_speed;
        let bottom_layer_time = fexp.exposure_time
            + fexp.lift_distance / fexp.lift_speed
            + fexp.retract_distance / fexp.retract_speed;
        layers.saturating_sub(self.first_layers) as f32 * layer_time
            + self.first_layers as f32 * bottom_layer_time
    }
}

impl Default for SliceConfig {
    fn default() -> Self {
        Self {
            format: Format::Ctb,

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
            lift_distance: Milimeters::new(5.0),
            lift_speed: (Milimeters::new(330.0) / Minutes::new(1.0)).convert(),
            retract_distance: Milimeters::new(5.0),
            retract_speed: (Milimeters::new(330.0) / Minutes::new(1.0)).convert(),
        }
    }
}

impl SliceConfig {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.format.serialize(ser);
        self.platform_resolution.serialize(ser);
        self.platform_size.map(|x| x.raw()).serialize(ser);
        ser.write_f32_be(self.slice_height.raw());
        self.exposure_config.serialize(ser);
        self.first_exposure_config.serialize(ser);
        ser.write_u32_be(self.first_layers);
        ser.write_u32_be(self.transition_layers);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Result<Self> {
        Ok(Self {
            format: Format::deserialize(des)?,
            platform_resolution: Vector2::deserialize(des),
            platform_size: Vector3::deserialize(des).map(Milimeters::new),
            slice_height: Milimeters::new(des.read_f32_be()),
            exposure_config: ExposureConfig::deserialize(des),
            first_exposure_config: ExposureConfig::deserialize(des),
            first_layers: des.read_u32_be(),
            transition_layers: des.read_u32_be(),
        })
    }
}

impl ExposureConfig {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_be(self.exposure_time.raw());
        ser.write_f32_be(self.lift_distance.raw());
        ser.write_f32_be(self.lift_speed.raw());
        ser.write_f32_be(self.retract_distance.raw());
        ser.write_f32_be(self.retract_speed.raw());
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            exposure_time: Seconds::new(des.read_f32_be()),
            lift_distance: Milimeters::new(des.read_f32_be()),
            lift_speed: CentimetersPerSecond::new(des.read_f32_be()),
            retract_distance: Milimeters::new(des.read_f32_be()),
            retract_speed: CentimetersPerSecond::new(des.read_f32_be()),
        }
    }
}
