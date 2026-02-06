use anyhow::{Result, ensure};

use chrono::Local;
use common::{
    misc::SliceResult,
    serde::{Serializer, SizedString, SliceDeserializer},
    units::Second,
};

use crate::{ENDING_STRING, Header, LayerContent};

pub struct File {
    pub header: Header,
    pub layers: Vec<LayerContent>,
}

impl File {
    pub fn new(header: Header, layers: Vec<LayerContent>) -> Self {
        Self { header, layers }
    }

    pub fn from_slice_result(result: SliceResult<LayerContent>) -> Self {
        let SliceResult {
            layers,
            slice_config,
        } = result;

        let exp = &slice_config.exposure_config;
        let fexp = &slice_config.first_exposure_config;

        let layer_time = exp.exposure_time
            + exp.lift_distance / exp.lift_speed
            + exp.retract_distance / exp.retract_speed;
        let bottom_layer_time = fexp.exposure_time
            + fexp.lift_distance / fexp.lift_speed
            + fexp.retract_distance / fexp.retract_speed;
        let total_time = (layers.len() as u32 - slice_config.first_layers) as f32 * layer_time
            + slice_config.first_layers as f32 * bottom_layer_time;

        Self::new(
            Header {
                x_resolution: slice_config.platform_resolution.x as u16,
                y_resolution: slice_config.platform_resolution.y as u16,
                x_size: slice_config.platform_size.x,
                y_size: slice_config.platform_size.y,
                z_size: slice_config.platform_size.z,

                layer_count: layers.len() as u32,
                printing_time: total_time.get::<Second>() as u32,
                layer_thickness: slice_config.slice_height,
                bottom_layers: slice_config.first_layers,
                transition_layers: slice_config.transition_layers as u16,

                exposure_time: slice_config.exposure_config.exposure_time,
                lift_distance: slice_config.exposure_config.lift_distance,
                lift_speed: slice_config.exposure_config.lift_speed.convert(),
                retract_distance: slice_config.exposure_config.retract_distance,
                retract_speed: slice_config.exposure_config.retract_speed.convert(),

                bottom_exposure_time: slice_config.first_exposure_config.exposure_time,
                bottom_lift_distance: slice_config.first_exposure_config.lift_distance,
                bottom_lift_speed: slice_config.first_exposure_config.lift_speed.convert(),
                bottom_retract_distance: slice_config.first_exposure_config.retract_distance,
                bottom_retract_speed: slice_config.first_exposure_config.retract_speed.convert(),

                file_time: SizedString::new(
                    Local::now()
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                        .as_bytes(),
                ),

                ..Default::default()
            },
            layers,
        )
    }
}

impl File {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.header.serialize(ser);
        for layer in &self.layers {
            layer.serialize(ser);
        }
        ser.write_bytes(ENDING_STRING);
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        let header = Header::deserialize(des)?;
        let mut layers = Vec::with_capacity(header.layer_count as usize);

        for _ in 0..header.layer_count {
            layers.push(LayerContent::deserialize(des)?);
        }

        ensure!(des.read_slice(ENDING_STRING.len()) == ENDING_STRING);
        Ok(Self { header, layers })
    }
}
