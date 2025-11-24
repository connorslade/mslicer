use std::fmt::Debug;

use anyhow::Result;

use common::serde::{Deserializer, Serializer};

use crate::{Section, crypto::decrypt_in_place};

#[derive(Debug)]
pub struct LayerRef {
    pub layer_offset: u32,
    pub page_number: u32,
    pub layer_table_size: u32, // Should alys be 0x58
}

pub struct Layer {
    pub table_size: u32,
    pub position_z: f32,
    pub exposure_time: f32,
    pub light_off_delay: f32,
    pub page_number: u32,
    pub lift_height: f32,
    pub lift_speed: f32,
    pub lift_height_2: f32,
    pub lift_speed_2: f32,
    pub retract_speed: f32,
    pub retract_height_2: f32,
    pub retract_speed_2: f32,
    pub rest_time_before_lift: f32,
    pub rest_time_after_lift: f32,
    pub rest_time_after_retract: f32,
    pub light_pwm: f32,
    pub data: Vec<u8>,
}

impl LayerRef {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        let this = Self {
            layer_offset: des.read_u32_le(),
            page_number: des.read_u32_le(),
            layer_table_size: des.read_u32_le(),
        };
        des.advance_by(4);
        Ok(this)
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(self.layer_offset);
        ser.write_u32_le(self.page_number);
        ser.write_u32_le(self.layer_table_size);
        ser.write_u32_le(0);
    }
}

impl Layer {
    pub fn deserialize(des: &mut Deserializer, xor_key: u32, layer: u32) -> Result<Self> {
        let table_size = des.read_u32_le();
        let position_z = des.read_f32_le();
        let exposure_time = des.read_f32_le();
        let light_off_delay = des.read_f32_le();
        let layer_offset = des.read_u32_le();
        let page_number = des.read_u32_le();

        let layer_size = des.read_u32_le();
        des.advance_by(4);
        let encrypted_data = Section::deserialize(des)?;

        let mut data = des.execute_at(layer_offset as usize, |des| {
            des.read_bytes(layer_size as usize).to_vec()
        });

        if encrypted_data.size > 0 {
            let (offset, length) = (encrypted_data.offset as usize, encrypted_data.size as usize);
            decrypt_in_place(&mut data[offset..offset + length]);
        }

        if xor_key > 0 {
            xor_cypher(&mut data, xor_key, layer);
        }

        Ok(Self {
            table_size,
            position_z,
            exposure_time,
            light_off_delay,
            page_number,
            lift_height: des.read_f32_le(),
            lift_speed: des.read_f32_le(),
            lift_height_2: des.read_f32_le(),
            lift_speed_2: des.read_f32_le(),
            retract_speed: des.read_f32_le(),
            retract_height_2: des.read_f32_le(),
            retract_speed_2: des.read_f32_le(),
            rest_time_before_lift: des.read_f32_le(),
            rest_time_after_lift: des.read_f32_le(),
            rest_time_after_retract: des.read_f32_le(),
            light_pwm: des.read_f32_le(),
            data,
        })
    }
}

fn xor_cypher(data: &mut [u8], seed: u32, layer: u32) {
    let init = seed.wrapping_mul(0x2D83CDAC).wrapping_add(0xD8A83423);
    let mut key = layer
        .wrapping_mul(0x1E1530CD)
        .wrapping_add(0xEC3D47CD)
        .wrapping_mul(init);

    let mut index = 0;
    for byte in data.iter_mut() {
        let k = (key >> (8 * index)) as u8;
        index += 1;

        if index & 3 == 0 {
            key = key.wrapping_add(init);
            index = 0;
        }

        *byte ^= k;
    }
}

impl Debug for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layer")
            .field("table_size", &self.table_size)
            .field("position_z", &self.position_z)
            .field("exposure_time", &self.exposure_time)
            .field("light_off_delay", &self.light_off_delay)
            .field("page_number", &self.page_number)
            .field("lift_height", &self.lift_height)
            .field("lift_speed", &self.lift_speed)
            .field("lift_height_2", &self.lift_height_2)
            .field("lift_speed_2", &self.lift_speed_2)
            .field("retract_speed", &self.retract_speed)
            .field("retract_height_2", &self.retract_height_2)
            .field("retract_speed_2", &self.retract_speed_2)
            .field("rest_time_before_lift", &self.rest_time_before_lift)
            .field("rest_time_after_lift", &self.rest_time_after_lift)
            .field("rest_time_after_retract", &self.rest_time_after_retract)
            .field("light_pwm", &self.light_pwm)
            .finish()
    }
}
