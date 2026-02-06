use std::fmt::Debug;

use anyhow::{Result, ensure};

use common::{
    serde::{Deserializer, Serializer, SliceDeserializer},
    units::{Milimeters, MilimetersPerMinute, Seconds},
};

use crate::{Section, crypto::decrypt_in_place};

#[derive(Debug)]
pub struct LayerRef {
    pub layer_offset: u32,
    pub page_number: u32,
}

pub struct Layer {
    pub position_z: Milimeters,
    pub exposure_time: Seconds,
    pub light_off_delay: Seconds,
    pub lift_height: Milimeters,
    pub lift_speed: MilimetersPerMinute,
    pub lift_height_2: Milimeters,
    pub lift_speed_2: MilimetersPerMinute,
    pub retract_speed: MilimetersPerMinute,
    pub retract_height_2: Milimeters,
    pub retract_speed_2: MilimetersPerMinute,
    pub rest_time_before_lift: Seconds,
    pub rest_time_after_lift: Seconds,
    pub rest_time_after_retract: Seconds,
    pub light_pwm: f32,
    pub data: Vec<u8>,
}

impl LayerRef {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        let this = Self {
            layer_offset: des.read_u32_le(),
            page_number: des.read_u32_le(),
        };
        ensure!(des.read_u32_le() == 0x58);
        des.advance_by(4);
        Ok(this)
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(self.layer_offset);
        ser.write_u32_le(self.page_number);
        ser.write_u32_le(0x58);
        ser.write_u32_le(0);
    }
}

impl Layer {
    pub fn deserialize(des: &mut SliceDeserializer, xor_key: u32, layer: u32) -> Result<Self> {
        let table_size = des.read_u32_le();
        ensure!(table_size == 0x58);

        let position_z = Milimeters::new(des.read_f32_le());
        let exposure_time = Seconds::new(des.read_f32_le());
        let light_off_delay = Seconds::new(des.read_f32_le());
        let layer_offset = des.read_u32_le();
        let _page_number = des.read_u32_le();

        let layer_size = des.read_u32_le();
        des.advance_by(4);
        let encrypted_data = Section::deserialize(des)?;

        let mut data = des.execute_at(layer_offset as usize, |des| {
            des.read_slice(layer_size as usize).to_vec()
        });

        if encrypted_data.size != 0 {
            let (offset, length) = (encrypted_data.offset as usize, encrypted_data.size as usize);
            decrypt_in_place(&mut data[offset..offset + length]);
        }

        if xor_key != 0 {
            xor_cypher(&mut data, xor_key, layer);
        }

        Ok(Self {
            position_z,
            exposure_time,
            light_off_delay,
            lift_height: Milimeters::new(des.read_f32_le()),
            lift_speed: MilimetersPerMinute::new(des.read_f32_le()),
            lift_height_2: Milimeters::new(des.read_f32_le()),
            lift_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
            retract_speed: MilimetersPerMinute::new(des.read_f32_le()),
            retract_height_2: Milimeters::new(des.read_f32_le()),
            retract_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
            rest_time_before_lift: Seconds::new(des.read_f32_le()),
            rest_time_after_lift: Seconds::new(des.read_f32_le()),
            rest_time_after_retract: Seconds::new(des.read_f32_le()),
            light_pwm: des.read_f32_le(),
            data,
        })
    }

    pub fn serialize<T: Serializer>(
        &self,
        ser: &mut T,
        xor_key: u32,
        page_number: u32,
        layer: u32,
    ) {
        ser.write_u32_le(0x58);
        ser.write_f32_le(self.position_z.raw());
        ser.write_f32_le(self.exposure_time.raw());
        ser.write_f32_le(self.light_off_delay.raw());
        let layer_offset = ser.reserve(4);
        ser.write_u32_le(page_number);
        ser.write_u32_le(self.data.len() as u32);
        ser.write_u32_le(0);
        Section::new(0, 0).serialize(ser); // TODO: Not sure if the data can always just be left unencrypted
        ser.write_f32_le(self.lift_height.raw());
        ser.write_f32_le(self.lift_speed.raw());
        ser.write_f32_le(self.lift_height_2.raw());
        ser.write_f32_le(self.lift_speed_2.raw());
        ser.write_f32_le(self.retract_speed.raw());
        ser.write_f32_le(self.retract_height_2.raw());
        ser.write_f32_le(self.retract_speed_2.raw());
        ser.write_f32_le(self.rest_time_before_lift.raw());
        ser.write_f32_le(self.rest_time_after_lift.raw());
        ser.write_f32_le(self.rest_time_after_retract.raw());
        ser.write_f32_le(self.light_pwm);
        ser.write_u32_le(0);

        let position = ser.pos();
        ser.execute_at(layer_offset, |ser| ser.write_u32_le(position as u32));
        ser.write_bytes(&self.data);

        if xor_key != 0 {
            let buffer = ser.view_mut(position, self.data.len());
            xor_cypher(buffer, xor_key, layer);
        }
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
            .field("position_z", &self.position_z)
            .field("exposure_time", &self.exposure_time)
            .field("light_off_delay", &self.light_off_delay)
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
