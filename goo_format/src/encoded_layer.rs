use common::{
    config::SliceConfig,
    misc::{EncodableLayer, Run},
};
use nalgebra::Vector2;

use crate::{LayerContent, layer_content::calculate_checksum};

pub struct LayerEncoder {
    data: Vec<u8>,
    last_value: u8,
}

pub struct LayerDecoder<'a> {
    data: &'a [u8],
    color: u8,
    offset: usize,
}

impl LayerEncoder {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            last_value: 0,
        }
    }

    pub fn add_run(&mut self, length: u64, value: u8) {
        // byte 0: aabbcccc
        // a => 0: full black, 1: full white, 2: small diff, 3: large diff
        // b => 0: 4 bit length, 1: 12 bit value, 2: 20 bit value, 3: 28 bit value
        // c => the first 4 bits of the value
        // byte 1-3: optional, the rest of the value

        let diff = value as i16 - self.last_value as i16;
        let chunk_type: u8 = match value {
            // Full black and full white are always encoded as is
            0x00 => 0b00,
            0xFF => 0b11,
            _ if !self.data.is_empty() && diff.abs() <= 15 => {
                // 0babcccc
                // a => 0: add diff, 1: sub diff
                // b => 0: length of 1, 1: length is next byte
                // c => the diff

                if length > 255 {
                    self.add_run(255, value);
                    self.add_run(length - 255, value);
                    return;
                }

                let byte_0 = (0b10 << 6)
                    | (((diff < 0) as u8) << 5)
                    | (((length != 1) as u8) << 4)
                    | (diff.unsigned_abs() as u8);
                self.data.push(byte_0);

                if length != 1 {
                    self.data.push(length as u8);
                }

                self.last_value = value;
                return;
            }
            _ => 0b01,
        };

        let chunk_length_size = match length {
            0x0000000..=0x000000F => 0b00,
            0x0000010..=0x0000FFF => 0b01,
            0x0001000..=0x00FFFFF => 0b10,
            0x0100000..=0xFFFFFFF => 0b11,
            _ => {
                self.add_run(0xFFFFFFF, value);
                self.add_run(length - 0xFFFFFFF, value);
                return;
            }
        };

        self.data
            .push((chunk_type << 6) | (chunk_length_size << 4) | (length as u8 & 0x0F));

        if chunk_type == 0b01 {
            self.data.push(value);
        }

        match chunk_length_size {
            1 => self.data.extend_from_slice(&[(length >> 4) as u8]),
            2 => self
                .data
                .extend_from_slice(&[(length >> 12) as u8, (length >> 4) as u8]),
            3 => self.data.extend_from_slice(&[
                (length >> 20) as u8,
                (length >> 12) as u8,
                (length >> 4) as u8,
            ]),
            _ => {}
        }

        self.last_value = value;
    }

    pub fn finish(self) -> (Vec<u8>, u8) {
        let checksum = calculate_checksum(&self.data);
        (self.data, checksum)
    }
}

impl EncodableLayer for LayerEncoder {
    type Output = LayerContent;

    fn new(_platform: Vector2<u32>) -> Self {
        Self::default()
    }

    fn add_run(&mut self, length: u64, value: u8) {
        self.add_run(length, value);
    }

    fn finish(self, layer: u64, slice_config: &SliceConfig) -> Self::Output {
        let (data, checksum) = self.finish();
        let layer_exposure = slice_config.exposure_config(layer);

        LayerContent {
            data,
            checksum,
            layer_position_z: slice_config.slice_height * (layer + 1) as f32,

            layer_exposure_time: layer_exposure.exposure_time,
            lift_distance: layer_exposure.lift_distance,
            lift_speed: layer_exposure.lift_speed.convert(),
            retract_distance: layer_exposure.retract_distance,
            retract_speed: layer_exposure.retract_speed.convert(),
            pause_position_z: slice_config.platform_size.z,
            ..Default::default()
        }
    }
}

impl<'a> LayerDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            color: 0,
            offset: 0,
        }
    }

    pub fn checksum(&self) -> u8 {
        calculate_checksum(self.data)
    }
}

impl Iterator for LayerDecoder<'_> {
    type Item = Run;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        let mut length = 0;
        let head = self.data[self.offset];

        // 0b00 -> All 0x00 pixels
        // 0b01 -> Gray between 0x01 to 0xFE (byte 1)
        // 0b10 -> Diff value from the previous pixel
        // 0b11 -> All 0xFF pixels
        let chunk_type = head >> 6;
        let chunk_length_size = head >> 4 & 0x03;
        match chunk_type {
            0b00 => self.color = 0,
            0b01 => {
                self.offset += 1;
                self.color = self.data[self.offset]
            }
            0b10 => {
                let diff_type = head >> 4 & 0x03;
                let diff_value = head & 0x0F;
                if diff_type & 0b01 == 0 {
                    length = 1;
                } else {
                    self.offset += 1;
                    length = self.data[self.offset] as u64;
                }
                if diff_type & 0b10 == 0 {
                    self.color += diff_value;
                } else {
                    self.color -= diff_value;
                }
            }
            0b11 => self.color = 0xFF,
            _ => unreachable!(),
        };

        if chunk_type != 0b10 {
            let base = (head & 0x0F) as u64;
            match chunk_length_size {
                0b00 => length = base,
                0b01 => {
                    length = base + ((self.data[self.offset + 1] as u64) << 4);
                    self.offset += 1;
                }
                0b10 => {
                    length = base
                        + ((self.data[self.offset + 1] as u64) << 12)
                        + ((self.data[self.offset + 2] as u64) << 4);
                    self.offset += 2;
                }
                0b11 => {
                    length = base
                        + ((self.data[self.offset + 1] as u64) << 20)
                        + ((self.data[self.offset + 2] as u64) << 12)
                        + ((self.data[self.offset + 3] as u64) << 4);
                    self.offset += 3;
                }
                _ => unreachable!(),
            };
        }
        self.offset += 1;

        Some(Run {
            length,
            value: self.color,
        })
    }
}

impl Default for LayerEncoder {
    fn default() -> Self {
        Self::new()
    }
}
