use std::fmt::Debug;

use anyhow::{Ok, Result};
use common::{
    serde::{Deserializer, Serializer, SliceDeserializer},
    units::{Milimeter, Milimeters, MilimetersPerMinute, Milliliters, Minute, Second, Seconds},
};

use crate::shared::{Section, read_string};

#[derive(Debug)]
pub struct PrintParameters {
    pub lift_height: Milimeters,
    pub lift_speed: MilimetersPerMinute,
    pub light_off_delay: Seconds,
    pub retract_speed: MilimetersPerMinute,

    pub bottom_lift_height: Milimeters,
    pub bottom_lift_speed: MilimetersPerMinute,
    pub bottom_light_off_delay: Seconds,

    pub volume: Milliliters,
    pub weight: f32, // grams (todo: make unit type?)
    pub cost: f32,
    pub bottom_layer_count: u32, // duplicated 4 sum reason?? (so close to writing a chitu rant)
}

#[derive(Debug)]
pub struct SlicerParameters {
    pub bottom_lift_height_2: Milimeters,
    pub bottom_lift_speed_2: MilimetersPerMinute,
    pub lift_height_2: Milimeters,
    pub lift_speed_2: MilimetersPerMinute,
    pub retract_height_2: Milimeters,
    pub retract_speed_2: MilimetersPerMinute,
    pub rest_time_after_lift: Seconds,
    pub machine_name: String,
    pub anti_alias_flag: u8, // 0x07 no AA, 0x0F AA
    pub per_layer_settings: u8,
    pub timestamp: u32,
    pub anti_alias_level: u32,
    pub software_version: u32,
    pub rest_time_after_retract: Seconds,
    pub rest_time_after_lift_2: Seconds,
    pub transition_layers: u32,
    pub ext: PrinterParametersExt,
}

#[derive(Debug)]
pub struct PrinterParametersExt {
    pub bottom_retract_speed: MilimetersPerMinute,
    pub bottom_retract_speed_2: MilimetersPerMinute,
    pub rest_time_after_retract: Seconds,
    pub rest_time_after_lift: Seconds,
    pub rest_time_before_lift: Seconds,
    pub bottom_retract_height_2: Milimeters,
    pub last_layer_idx: u32,
    pub disclaimer: String,
}

impl PrintParameters {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            bottom_lift_height: Milimeters::new(des.read_f32_le()),
            bottom_lift_speed: MilimetersPerMinute::new(des.read_f32_le()),
            lift_height: Milimeters::new(des.read_f32_le()),
            lift_speed: MilimetersPerMinute::new(des.read_f32_le()),
            retract_speed: MilimetersPerMinute::new(des.read_f32_le()),
            volume: Milliliters::new(des.read_f32_le()),
            weight: des.read_f32_le(),
            cost: des.read_f32_le(),
            bottom_light_off_delay: Seconds::new(des.read_f32_le()),
            light_off_delay: Seconds::new(des.read_f32_le()),
            bottom_layer_count: des.read_u32_le(),
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_le(self.bottom_lift_height.get::<Milimeter>());
        ser.write_f32_le(self.bottom_lift_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.lift_height.get::<Milimeter>());
        ser.write_f32_le(self.lift_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.retract_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.volume.get::<Milimeter>());
        ser.write_f32_le(self.weight);
        ser.write_f32_le(self.cost);
        ser.write_f32_le(self.bottom_light_off_delay.get::<Second>());
        ser.write_f32_le(self.light_off_delay.get::<Second>());
        ser.write_u32_le(self.bottom_layer_count);
        ser.reserve(4 * 4);
    }
}

impl SlicerParameters {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            bottom_lift_height_2: Milimeters::new(des.read_f32_le()),
            bottom_lift_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
            lift_height_2: Milimeters::new(des.read_f32_le()),
            lift_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
            retract_height_2: Milimeters::new(des.read_f32_le()),
            retract_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
            rest_time_after_lift: Seconds::new(des.read_f32_le()),
            machine_name: {
                let section = Section::deserialize(des)?;
                read_string(des, section).into_owned()
            },
            anti_alias_flag: des.read_u8(),
            per_layer_settings: {
                des.advance_by(2);
                des.read_u8()
            },
            timestamp: des.read_u32_le(),
            anti_alias_level: des.read_u32_le(),
            software_version: des.read_u32_le(),
            rest_time_after_retract: Seconds::new(des.read_f32_le()),
            rest_time_after_lift_2: Seconds::new(des.read_f32_le()),
            transition_layers: des.read_u32_le(),
            ext: {
                let ext = des.read_u32_le() as usize;
                des.execute_at(ext, |des| PrinterParametersExt::deserialize(des))?
            },
            // 8 bytes of padding
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_le(self.bottom_lift_height_2.get::<Milimeter>());
        ser.write_f32_le(self.bottom_lift_speed_2.get::<Milimeter, Minute>());
        ser.write_f32_le(self.lift_height_2.get::<Milimeter>());
        ser.write_f32_le(self.lift_speed_2.get::<Milimeter, Minute>());
        ser.write_f32_le(self.retract_height_2.get::<Milimeter>());
        ser.write_f32_le(self.retract_speed_2.get::<Milimeter, Minute>());
        ser.write_f32_le(self.rest_time_after_lift.get::<Second>());
        let machine_name = ser.reserve(8);
        ser.write_u8(self.anti_alias_flag);
        ser.reserve(2);
        ser.write_u8(self.per_layer_settings);
        ser.write_u32_le(self.timestamp);
        ser.write_u32_le(self.anti_alias_level);
        ser.write_u32_le(self.software_version);
        ser.write_f32_le(self.rest_time_after_retract.get::<Second>());
        ser.write_f32_le(self.rest_time_after_lift_2.get::<Second>());
        ser.write_u32_le(self.transition_layers);
        let ext_offset = ser.reserve(4);
        ser.reserve(4 * 2);

        let section = Section::new(ser.pos(), self.machine_name.len());
        ser.execute_at(machine_name, |ser| section.serialize(ser));
        ser.write_bytes(self.machine_name.as_bytes());

        let offset = ser.pos() as u32;
        ser.execute_at(ext_offset, |ser| ser.write_u32_le(offset));
        self.ext.serialize(ser);
    }
}

impl PrinterParametersExt {
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        {
            Ok(Self {
                bottom_retract_speed: MilimetersPerMinute::new(des.read_f32_le()),
                bottom_retract_speed_2: MilimetersPerMinute::new(des.read_f32_le()),
                rest_time_after_retract: {
                    des.advance_by(4 * 4);
                    Seconds::new(des.read_f32_le())
                },
                rest_time_after_lift: Seconds::new(des.read_f32_le()),
                rest_time_before_lift: Seconds::new(des.read_f32_le()),
                bottom_retract_height_2: Milimeters::new(des.read_f32_le()),
                last_layer_idx: {
                    des.advance_by(4 * 3);
                    des.read_u32_le()
                },
                disclaimer: {
                    des.advance_by(4 * 4);
                    let section = Section::deserialize(des)?;
                    read_string(des, section).into_owned()
                },
            })
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_f32_le(self.bottom_retract_speed.get::<Milimeter, Minute>());
        ser.write_f32_le(self.bottom_retract_speed_2.get::<Milimeter, Minute>());
        ser.reserve(4);
        ser.write_f32_le(4.0);
        ser.reserve(4);
        ser.write_f32_le(4.0);
        ser.write_f32_le(self.rest_time_after_retract.get::<Second>());
        ser.write_f32_le(self.rest_time_after_lift.get::<Second>());
        ser.write_f32_le(self.rest_time_before_lift.get::<Second>());
        ser.write_f32_le(self.bottom_retract_height_2.get::<Milimeter>());
        ser.write_f32_le(2955.996);
        ser.reserve(4);
        ser.write_u32_le(5);
        ser.write_u32_le(self.last_layer_idx);
        ser.reserve(4 * 4);
        let disclaimer = ser.reserve(8);
        ser.reserve(384 + 4);

        let section = Section::new(ser.pos(), self.disclaimer.len());
        ser.execute_at(disclaimer, |ser| section.serialize(ser));
        ser.write_bytes(self.disclaimer.as_bytes());
    }
}
