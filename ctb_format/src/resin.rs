use anyhow::Result;
use nalgebra::Vector4;

use common::serde::{Deserializer, Serializer};

use crate::{Section, read_string};

#[derive(Debug)]
pub struct ResinParameters {
    pub resin_color: Vector4<u8>,
    pub machine_name: String,
    pub resin_type: String,
    pub resin_name: String,
    pub resin_density: f32,
}

impl ResinParameters {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        des.advance_by(4);

        let color_b = des.read_u8();
        let color_g = des.read_u8();
        let color_r = des.read_u8();
        let color_a = des.read_u8();

        let machine_name_address = des.read_u32_le();
        let resin_type = Section::deserialize_rev(des)?;
        let resin_name = Section::deserialize_rev(des)?;
        let machine_name = Section {
            size: des.read_u32_le(),
            offset: machine_name_address,
        };
        let resin_density = des.read_f32_le();

        Ok(Self {
            resin_color: Vector4::new(color_r, color_g, color_b, color_a),
            machine_name: read_string(des, machine_name),
            resin_type: read_string(des, resin_type),
            resin_name: read_string(des, resin_name),
            resin_density,
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_be(0);
        ser.write_u8(self.resin_color.z);
        ser.write_u8(self.resin_color.y);
        ser.write_u8(self.resin_color.x);
        ser.write_u8(self.resin_color.w);
        let machine_name_address = ser.reserve(4);
        let resin_type = ser.reserve(8);
        let resin_name = ser.reserve(8);
        let machine_name_size = ser.reserve(4);
        ser.write_f32_le(self.resin_density);
        ser.write_u32_le(0);

        let machine_name_bytes = self.machine_name.as_bytes();
        let machine_name_offset = ser.pos();
        ser.write_bytes(machine_name_bytes);
        ser.execute_at(machine_name_address, |ser| {
            ser.write_u32_le(machine_name_offset as u32);
        });
        ser.execute_at(machine_name_size, |ser| {
            ser.write_u32_le(machine_name_bytes.len() as u32)
        });

        serialize_string(ser, resin_type, &self.resin_type);
        serialize_string(ser, resin_name, &self.resin_name);
    }
}

fn serialize_string<T: Serializer>(ser: &mut T, offset: usize, string: &str) {
    let string_bytes = string.as_bytes();
    let string_offset = ser.pos();
    ser.write_bytes(string_bytes);
    ser.execute_at(offset, |ser| {
        Section::new(string_offset, string_bytes.len()).serialize_rev(ser);
    });
}
