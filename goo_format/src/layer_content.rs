use anyhow::{Result, ensure};

use common::{
    serde::{Deserializer, Serializer, SliceDeserializer},
    units::{Milimeters, MilimetersPerMinute, Seconds},
};

use crate::DELIMITER;

pub struct LayerContent {
    /// If printing should be paused on current layer.
    pub pause: bool,
    /// The Z position to to if paused.
    pub pause_position_z: Milimeters,
    /// The Z position of the layer.
    /// `(layer_height * (i + 1))`.
    pub layer_position_z: Milimeters,
    /// Exposure time for the layer.
    pub layer_exposure_time: Seconds,
    /// Time to wait after the layer is done when exposure delay mode is 0, in seconds.
    pub layer_off_time: f32,
    /// Time to wait before lifting the platform when exposure delay mode is 1, in seconds.
    pub before_lift_time: f32,
    /// Time to wait after lifting the platform when exposure delay mode is 1, in seconds.
    pub after_lift_time: f32,
    /// Time to wait after retracting the platform when exposure delay mode is 1, in seconds.
    pub after_retract_time: f32,
    /// Distance to lift the platform.
    pub lift_distance: Milimeters,
    /// Speed to lift the platform.
    pub lift_speed: MilimetersPerMinute,
    /// Distance to lift the platform a second time, in mm.
    pub second_lift_distance: f32,
    /// Speed to lift the platform a second time, in mm/min.
    pub second_lift_speed: f32,
    /// Distance to retract the platform.
    pub retract_distance: Milimeters,
    /// Speed to retract the platform.
    pub retract_speed: MilimetersPerMinute,
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
        ser.write_f32_be(self.pause_position_z.raw());
        ser.write_f32_be(self.layer_position_z.raw());
        ser.write_f32_be(self.layer_exposure_time.raw());
        ser.write_f32_be(self.layer_off_time);
        ser.write_f32_be(self.before_lift_time);
        ser.write_f32_be(self.after_lift_time);
        ser.write_f32_be(self.after_retract_time);
        ser.write_f32_be(self.lift_distance.raw());
        ser.write_f32_be(self.lift_speed.raw());
        ser.write_f32_be(self.second_lift_distance);
        ser.write_f32_be(self.second_lift_speed);
        ser.write_f32_be(self.retract_distance.raw());
        ser.write_f32_be(self.retract_speed.raw());
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

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            pause: des.read_u16_be() != 0,
            pause_position_z: Milimeters::new(des.read_f32_be()),
            layer_position_z: Milimeters::new(des.read_f32_be()),
            layer_exposure_time: Seconds::new(des.read_f32_be()),
            layer_off_time: des.read_f32_be(),
            before_lift_time: des.read_f32_be(),
            after_lift_time: des.read_f32_be(),
            after_retract_time: des.read_f32_be(),
            lift_distance: Milimeters::new(des.read_f32_be()),
            lift_speed: MilimetersPerMinute::new(des.read_f32_be()),
            second_lift_distance: des.read_f32_be(),
            second_lift_speed: des.read_f32_be(),
            retract_distance: Milimeters::new(des.read_f32_be()),
            retract_speed: MilimetersPerMinute::new(des.read_f32_be()),
            second_retract_distance: des.read_f32_be(),
            second_retract_speed: des.read_f32_be(),
            light_pwm: des.read_u16_be().min(255) as u8,
            data: {
                ensure!(des.read_slice(2) == DELIMITER);
                let data_len = des.read_u32_be() as usize - 2;
                ensure!(des.read_u8() == 0x55);
                des.read_slice(data_len).to_vec() // ehhh its fiiiine (its not fine)
            },
            checksum: {
                let checksum = des.read_u8();
                ensure!(des.read_slice(2) == DELIMITER);
                checksum
            },
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
