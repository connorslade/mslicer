use anyhow::Result;
use nalgebra::{Vector2, Vector3};

use crate::{
    format::Format,
    serde::{Deserializer, SerdeExt, Serializer},
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SliceConfig {
    pub format: Format,

    pub platform_resolution: Vector2<u32>,
    pub platform_size: Vector3<f32>, // mm
    pub slice_height: f32,           // mm

    pub exposure_config: ExposureConfig,
    pub first_exposure_config: ExposureConfig,
    pub first_layers: u32,
    pub transition_layers: u32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExposureConfig {
    pub exposure_time: f32,    // s
    pub lift_distance: f32,    // mm
    pub lift_speed: f32,       // cm/min
    pub retract_distance: f32, // mm
    pub retract_speed: f32,    // cm/min
}

impl SliceConfig {
    pub fn exposure_config(&self, layer: u64) -> &ExposureConfig {
        if (layer as u32) < self.first_layers {
            &self.first_exposure_config
        } else {
            &self.exposure_config
        }
    }
}

impl Default for SliceConfig {
    fn default() -> Self {
        Self {
            format: Format::Ctb,

            platform_resolution: Vector2::new(11_520, 5_120),
            platform_size: Vector3::new(218.88, 122.904, 260.0),
            slice_height: 0.05,
            exposure_config: ExposureConfig {
                exposure_time: 3.0,
                ..Default::default()
            },
            first_exposure_config: ExposureConfig {
                exposure_time: 30.0,
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
            exposure_time: 3.0,
            lift_distance: 5.0,
            lift_speed: 1.1,
            retract_distance: 5.0,
            retract_speed: 2.5,
        }
    }
}

impl SliceConfig {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.format.serialize(ser);
        self.platform_resolution.serialize(ser);
        self.platform_size.serialize(ser);
        ser.write_f32_be(self.slice_height);
        self.exposure_config.serialize(ser);
        self.first_exposure_config.serialize(ser);
        ser.write_u32_be(self.first_layers);
        ser.write_u32_be(self.transition_layers);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Result<Self> {
        Ok(Self {
            format: Format::deserialize(des)?,
            platform_resolution: Vector2::deserialize(des),
            platform_size: Vector3::deserialize(des),
            slice_height: des.read_f32_be(),
            exposure_config: ExposureConfig::deserialize(des),
            first_exposure_config: ExposureConfig::deserialize(des),
            first_layers: des.read_u32_be(),
            transition_layers: des.read_u32_be(),
        })
    }
}

impl ExposureConfig {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_be(self.exposure_time);
        ser.write_f32_be(self.lift_distance);
        ser.write_f32_be(self.lift_speed);
        ser.write_f32_be(self.retract_distance);
        ser.write_f32_be(self.retract_speed);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            exposure_time: des.read_f32_be(),
            lift_distance: des.read_f32_be(),
            lift_speed: des.read_f32_be(),
            retract_distance: des.read_f32_be(),
            retract_speed: des.read_f32_be(),
        }
    }
}
