use anyhow::{ensure, Result};

use common::serde::{Deserializer, Serializer};

use crate::DELIMITER;

pub struct LayerContent {
    pub pause_flag: u16,
    pub pause_position_z: f32,
    pub layer_position_z: f32,
    pub layer_exposure_time: f32,
    pub layer_off_time: f32,
    pub before_lift_time: f32,
    pub after_lift_time: f32,
    pub after_retract_time: f32,
    pub lift_distance: f32,
    pub lift_speed: f32,
    pub second_lift_distance: f32,
    pub second_lift_speed: f32,
    pub retract_distance: f32,
    pub retract_speed: f32,
    pub second_retract_distance: f32,
    pub second_retract_speed: f32,
    pub light_pwm: u16,
    pub data: Vec<u8>,
    pub checksum: u8,
}

impl LayerContent {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u16(self.pause_flag);
        ser.write_f32(self.pause_position_z);
        ser.write_f32(self.layer_position_z);
        ser.write_f32(self.layer_exposure_time);
        ser.write_f32(self.layer_off_time);
        ser.write_f32(self.before_lift_time);
        ser.write_f32(self.after_lift_time);
        ser.write_f32(self.after_retract_time);
        ser.write_f32(self.lift_distance);
        ser.write_f32(self.lift_speed);
        ser.write_f32(self.second_lift_distance);
        ser.write_f32(self.second_lift_speed);
        ser.write_f32(self.retract_distance);
        ser.write_f32(self.retract_speed);
        ser.write_f32(self.second_retract_distance);
        ser.write_f32(self.second_retract_speed);
        ser.write_u16(self.light_pwm);
        ser.write_u32(self.data.len() as u32 + 2);
        ser.write_bytes(&[0x55]);
        ser.write_bytes(&self.data);
        ser.write_u8(calculate_checksum(&self.data));
        ser.write_bytes(DELIMITER);
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        let mut des = Deserializer::new(buf);

        let pause_flag = des.read_u16();
        let pause_position_z = des.read_f32();
        let layer_position_z = des.read_f32();
        let layer_exposure_time = des.read_f32();
        let layer_off_time = des.read_f32();
        let before_lift_time = des.read_f32();
        let after_lift_time = des.read_f32();
        let after_retract_time = des.read_f32();
        let lift_distance = des.read_f32();
        let lift_speed = des.read_f32();
        let second_lift_distance = des.read_f32();
        let second_lift_speed = des.read_f32();
        let retract_distance = des.read_f32();
        let retract_speed = des.read_f32();
        let second_retract_distance = des.read_f32();
        let second_retract_speed = des.read_f32();
        let light_pwm = des.read_u16();
        let data_len = des.read_u32() as usize - 2;
        ensure!(des.read_u8() == 0x55);
        let data = des.read_bytes(data_len);
        let checksum = des.read_u8();
        ensure!(des.read_bytes(2) == DELIMITER);

        Ok(Self {
            pause_flag,
            pause_position_z,
            layer_position_z,
            layer_exposure_time,
            layer_off_time,
            before_lift_time,
            after_lift_time,
            after_retract_time,
            lift_distance,
            lift_speed,
            second_lift_distance,
            second_lift_speed,
            retract_distance,
            retract_speed,
            second_retract_distance,
            second_retract_speed,
            light_pwm,
            data: data.to_vec(), // ehhh its fiiiine (its not fine)
            checksum,
        })
    }
}

pub fn calculate_checksum(data: &[u8]) -> u8 {
    let mut out = 0u8;
    for &byte in data {
        out = out.wrapping_add(byte);
    }
    !out
}
