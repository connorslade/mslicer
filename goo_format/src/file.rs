use anyhow::{ensure, Result};

use common::{
    misc::SliceResult,
    serde::{Deserializer, Serializer},
};

use crate::{HeaderInfo, LayerContent, LayerEncoder, ENDING_STRING};

pub struct File {
    pub header: HeaderInfo,
    pub layers: Vec<LayerContent>,
}

impl File {
    pub fn new(header: HeaderInfo, layers: Vec<LayerContent>) -> Self {
        Self { header, layers }
    }

    pub fn from_slice_result(result: SliceResult) -> Self {
        let slice_config = result.slice_config;
        let layers = result
            .layers
            .into_iter()
            .enumerate()
            .map(|(idx, layer)| {
                let mut encoder = LayerEncoder::new();

                for run in layer.runs() {
                    encoder.add_run(run.length, run.value);
                }

                let (data, checksum) = encoder.finish();
                let layer_exposure = if (idx as u32) < slice_config.first_layers {
                    &slice_config.first_exposure_config
                } else {
                    &slice_config.exposure_config
                };

                LayerContent {
                    data,
                    checksum,
                    layer_position_z: slice_config.slice_height * (idx + 1) as f32,

                    layer_exposure_time: layer_exposure.exposure_time,
                    lift_distance: layer_exposure.lift_distance,
                    lift_speed: layer_exposure.lift_speed,
                    retract_distance: layer_exposure.retract_distance,
                    retract_speed: layer_exposure.retract_speed,
                    pause_position_z: slice_config.platform_size.z,
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();

        let layer_time = slice_config.exposure_config.exposure_time
            + slice_config.exposure_config.lift_distance / slice_config.exposure_config.lift_speed;
        let bottom_layer_time = slice_config.first_exposure_config.exposure_time
            + slice_config.first_exposure_config.lift_distance
                / slice_config.first_exposure_config.lift_speed;
        let total_time = (layers.len() as u32 - slice_config.first_layers) as f32 * layer_time
            + slice_config.first_layers as f32 * bottom_layer_time;

        Self::new(
            HeaderInfo {
                x_resolution: slice_config.platform_resolution.x as u16,
                y_resolution: slice_config.platform_resolution.y as u16,
                x_size: slice_config.platform_size.x,
                y_size: slice_config.platform_size.y,

                layer_count: layers.len() as u32,
                printing_time: total_time as u32,
                layer_thickness: slice_config.slice_height,
                bottom_layers: slice_config.first_layers,
                transition_layers: slice_config.first_layers as u16 + 1,

                exposure_time: slice_config.exposure_config.exposure_time,
                lift_distance: slice_config.exposure_config.lift_distance,
                lift_speed: slice_config.exposure_config.lift_speed,
                retract_distance: slice_config.exposure_config.retract_distance,
                retract_speed: slice_config.exposure_config.retract_speed,

                bottom_exposure_time: slice_config.first_exposure_config.exposure_time,
                bottom_lift_distance: slice_config.first_exposure_config.lift_distance,
                bottom_lift_speed: slice_config.first_exposure_config.lift_speed,
                bottom_retract_distance: slice_config.first_exposure_config.retract_distance,
                bottom_retract_speed: slice_config.first_exposure_config.retract_speed,

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

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        let mut des = Deserializer::new(buf);

        let header = HeaderInfo::deserialize(&mut des)?;
        let mut layers = Vec::with_capacity(header.layer_count as usize);

        for _ in 0..header.layer_count {
            layers.push(LayerContent::deserialize(&mut des)?);
        }

        ensure!(des.read_bytes(ENDING_STRING.len()) == ENDING_STRING);
        Ok(Self { header, layers })
    }
}
