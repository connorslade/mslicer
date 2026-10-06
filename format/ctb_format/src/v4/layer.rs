use std::fmt::Debug;

use anyhow::{Ok, Result, ensure};
use common::{
    serde::{Deserializer, Serializer, SliceDeserializer},
    slice::{self, ExposureConfig},
    units::{Milimeter, Milimeters, MilimetersPerMinute, Minute, Second, Seconds},
};

use crate::shared::{LayerDecoder, PAGE_SIZE, xor_cypher};

pub const LAYER_DEF_SIZE: usize = 0x24;
pub const LAYER_DEF_EXT_SIZE: usize = 0x54;

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

impl Layer {
    pub fn deserialize(des: &mut SliceDeserializer, key: u32, layer: u32) -> Result<Self> {
        des.advance_by(4 * 3);
        let address = des.read_u32_le() as usize;
        let size = des.read_u32_le() as usize;
        let page = des.read_u32_le() as usize;
        ensure!(des.read_u32_le() == LAYER_DEF_EXT_SIZE as u32);
        des.advance_by(8);

        let address = page * PAGE_SIZE as usize + address;
        let mut data = des.execute_at(address, |des| des.read_bytes(size).into_owned());
        (key != 0).then(|| xor_cypher(&mut data, key, layer));

        des.execute_at(address - LAYER_DEF_EXT_SIZE, |des| {
            let position_z = Milimeters::new(des.read_f32_le());
            let exposure_time = Seconds::new(des.read_f32_le());
            let light_off_delay = Seconds::new(des.read_f32_le());
            des.advance_by(LAYER_DEF_SIZE - 4 * 3);
            ensure!(des.read_u32_le() as usize == LAYER_DEF_EXT_SIZE + size);

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
        })
    }

    pub fn serialize_ref<T: Serializer>(&self, ser: &mut T, page: u32, offset: u32) {
        ser.write_f32_le(self.position_z.get::<Milimeter>());
        ser.write_f32_le(self.exposure_time.get::<Second>());
        ser.write_f32_le(self.light_off_delay.get::<Second>());
        ser.write_u32_le(offset);
        ser.write_u32_le(self.data.len() as u32);
        ser.write_u32_le(page);
        ser.write_u32_le(LAYER_DEF_EXT_SIZE as u32);
        ser.reserve(8);
    }

    pub fn serialize_ext<T: Serializer>(&self, ser: &mut T, page: u32, offset: u32) {
        self.serialize_ref(ser, page, offset);
        ser.write_u32_le((LAYER_DEF_EXT_SIZE + self.data.len()) as u32);
        ser.write_f32_le(self.lift_height.get::<Milimeter>());
        ser.write_f32_le(self.lift_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.lift_height_2.get::<Milimeter>());
        ser.write_f32_le(self.lift_speed_2.get::<Milimeter, Minute>());
        ser.write_f32_le(self.retract_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.retract_height_2.get::<Milimeter>());
        ser.write_f32_le(self.retract_speed_2.get::<Milimeter, Minute>());
        ser.write_f32_le(self.rest_time_before_lift.get::<Second>());
        ser.write_f32_le(self.rest_time_after_lift.get::<Second>());
        ser.write_f32_le(self.rest_time_after_retract.get::<Second>());
        ser.write_f32_le(self.light_pwm);
    }

    pub fn into_layer(&self) -> slice::Layer {
        let data = LayerDecoder::new(&self.data).collect();
        let exposure = ExposureConfig {
            exposure_time: self.exposure_time,
            exposure_delay: self.rest_time_after_retract,
            pwm: self.light_pwm as u8,
            lift_distance: self.lift_height,
            lift_speed: self.lift_speed.convert(),
            retract_speed: self.retract_speed.convert(),
        };
        slice::Layer::new(data, self.position_z, exposure)
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
