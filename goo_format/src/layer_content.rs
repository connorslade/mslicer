use anyhow::{ensure, Result};

use common::serde::{Deserializer, Serializer};

use crate::DELIMITER;

pub struct LayerContent {
    /// If printing should be paused on current layer.
    pub pause: bool,
    /// The Z position to to if paused, in mm.
    pub pause_position_z: f32,
    /// The Z position of the layer, in mm.
    /// `(layer_height * (i + 1))`.
    pub layer_position_z: f32,
    /// Exposure time for the layer, in seconds.
    pub layer_exposure_time: f32,
    /// Time to wait after the layer is done when exposure delay mode is 0, in seconds.
    pub layer_off_time: f32,
    /// Time to wait before lifting the platform when exposure delay mode is 1, in seconds.
    pub before_lift_time: f32,
    /// Time to wait after lifting the platform when exposure delay mode is 1, in seconds.
    pub after_lift_time: f32,
    /// Time to wait after retracting the platform when exposure delay mode is 1, in seconds.
    pub after_retract_time: f32,
    /// Distance to lift the platform, in mm.
    pub lift_distance: f32,
    /// Speed to lift the platform, in mm/min.
    pub lift_speed: f32,
    /// Distance to lift the platform a second time, in mm.
    pub second_lift_distance: f32,
    /// Speed to lift the platform a second time, in mm/min.
    pub second_lift_speed: f32,
    /// Distance to retract the platform, in mm.
    pub retract_distance: f32,
    /// Speed to retract the platform, in mm/min.
    pub retract_speed: f32,
    /// Distance to retract the platform a second time, in mm.
    pub second_retract_distance: f32,
    /// Speed to retract the platform a second time, in mm/min.
    pub second_retract_speed: f32,
    /// Brightness of the light, 0-255.
    pub light_pwm: u8,
    /// The actual layer data, run length encoded with [`goo_format::LayerEncoder`].
    pub data: Vec<u8>,
    /// Negative wrapping sum of all bytes in `data`.
    pub checksum: u8,
}

impl LayerContent {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u16_be(self.pause as u16);
        ser.write_f32_be(self.pause_position_z);
        ser.write_f32_be(self.layer_position_z);
        ser.write_f32_be(self.layer_exposure_time);
        ser.write_f32_be(self.layer_off_time);
        ser.write_f32_be(self.before_lift_time);
        ser.write_f32_be(self.after_lift_time);
        ser.write_f32_be(self.after_retract_time);
        ser.write_f32_be(self.lift_distance);
        ser.write_f32_be(self.lift_speed);
        ser.write_f32_be(self.second_lift_distance);
        ser.write_f32_be(self.second_lift_speed);
        ser.write_f32_be(self.retract_distance);
        ser.write_f32_be(self.retract_speed);
        ser.write_f32_be(self.second_retract_distance);
        ser.write_f32_be(self.second_retract_speed);
        ser.write_u16_be(self.light_pwm as u16);
        ser.write_bytes(DELIMITER);
        ser.write_u32_be(self.data.len() as u32 + 2);
        ser.write_bytes(&[0x55]);
        ser.write_bytes(&self.data);
        ser.write_u8(calculate_checksum(&self.data));
        ser.write_bytes(DELIMITER);
    }

    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        let pause_flag = des.read_u16_be();
        let pause_position_z = des.read_f32_be();
        let layer_position_z = des.read_f32_be();
        let layer_exposure_time = des.read_f32_be();
        let layer_off_time = des.read_f32_be();
        let before_lift_time = des.read_f32_be();
        let after_lift_time = des.read_f32_be();
        let after_retract_time = des.read_f32_be();
        let lift_distance = des.read_f32_be();
        let lift_speed = des.read_f32_be();
        let second_lift_distance = des.read_f32_be();
        let second_lift_speed = des.read_f32_be();
        let retract_distance = des.read_f32_be();
        let retract_speed = des.read_f32_be();
        let second_retract_distance = des.read_f32_be();
        let second_retract_speed = des.read_f32_be();
        let light_pwm = des.read_u16_be().min(255) as u8;
        ensure!(des.read_bytes(2) == DELIMITER);
        let data_len = des.read_u32_be() as usize - 2;
        ensure!(des.read_u8() == 0x55);
        let data = des.read_bytes(data_len);
        let checksum = des.read_u8();
        ensure!(des.read_bytes(2) == DELIMITER);

        Ok(Self {
            pause: pause_flag != 0,
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
