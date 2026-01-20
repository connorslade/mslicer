use common::{
    config::SliceConfig,
    misc::{EncodableLayer, Run},
};

use crate::layer::Layer;

pub struct LayerDecoder<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> LayerDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl Iterator for LayerDecoder<'_> {
    type Item = Run;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        let mut value = self.data[self.offset];
        let mut length = 1;

        if value & 0x80 != 0 {
            value &= 0x7F;
            self.offset += 1;

            let next = self.data[self.offset] as u64;
            if next & 0x80 == 0 {
                length = next;
            } else if next & 0xC0 == 0x80 {
                length = ((next & 0x3F) << 8) + self.data[self.offset + 1] as u64;
                self.offset += 1;
            } else if next & 0xE0 == 0xC0 {
                length = ((next & 0x1F) << 16)
                    + ((self.data[self.offset + 1] as u64) << 8)
                    + self.data[self.offset + 2] as u64;
                self.offset += 2;
            } else if next & 0xF0 == 0xE0 {
                length = ((next & 0xF) << 24)
                    + ((self.data[self.offset + 1] as u64) << 16)
                    + ((self.data[self.offset + 2] as u64) << 8)
                    + self.data[self.offset + 3] as u64;
                self.offset += 3;
            } else {
                self.offset = self.data.len();
                return None;
            }
        }

        self.offset += 1;
        if value != 0 {
            value = (value << 1) | 1;
        }

        Some(Run { length, value })
    }
}

#[derive(Default)]
pub struct LayerEncoder {
    data: Vec<u8>,
}

impl LayerEncoder {
    pub fn add_run(&mut self, length: u64, value: u8) {
        if length == 0 {
            return;
        }

        self.data.push((value >> 1) | (0x80 * (length > 1) as u8));

        if length <= 1 {
            // pass
        } else if length <= 0x7F {
            self.data.push(length as u8);
        } else if length <= 0x3FFF {
            self.data.push((length >> 8) as u8 | 0x80);
            self.data.push(length as u8);
        } else if length <= 0x1FFFFF {
            self.data.push((length >> 16) as u8 | 0xc0);
            self.data.push((length >> 8) as u8);
            self.data.push(length as u8);
        } else if length <= 0xFFFFFFF {
            self.data.push((length >> 24) as u8 | 0xe0);
            self.data.push((length >> 16) as u8);
            self.data.push((length >> 8) as u8);
            self.data.push(length as u8);
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl EncodableLayer for LayerEncoder {
    type Output = Layer;

    fn add_run(&mut self, length: u64, value: u8) {
        self.add_run(length, value);
    }

    fn finish(self, layer: u64, config: &SliceConfig) -> Self::Output {
        let layer_exposure = config.exposure_config(layer);
        // note that retract_distance is not used...

        Layer {
            position_z: config.slice_height * (layer + 1) as f32,
            exposure_time: layer_exposure.exposure_time,
            light_off_delay: 0.0,
            lift_height: layer_exposure.lift_distance,
            lift_speed: layer_exposure.lift_speed,
            lift_height_2: 0.0,
            lift_speed_2: 0.0,
            retract_speed: layer_exposure.retract_speed,
            retract_height_2: 0.0,
            retract_speed_2: 0.0,
            rest_time_before_lift: 0.0,
            rest_time_after_lift: 0.0,
            rest_time_after_retract: 1.0,
            light_pwm: 255.0,
            data: self.data,
        }
    }
}
