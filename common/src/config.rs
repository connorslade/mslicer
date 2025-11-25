use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};

use crate::format::Format;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SliceConfig {
    pub format: Format,

    pub platform_resolution: Vector2<u32>,
    pub platform_size: Vector3<f32>,
    pub slice_height: f32,

    pub exposure_config: ExposureConfig,
    pub first_exposure_config: ExposureConfig,
    pub first_layers: u32,
    pub transition_layers: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExposureConfig {
    pub exposure_time: f32,
    pub lift_distance: f32,
    pub lift_speed: f32,
    pub retract_distance: f32,
    pub retract_speed: f32,
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
            lift_speed: 65.0,
            retract_distance: 5.0,
            retract_speed: 150.0,
        }
    }
}
