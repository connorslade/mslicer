use anyhow::Result;
use nalgebra::Vector4;

use common::serde::Deserializer;

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
        let resin_type = Section::deserialize(des)?;
        let resin_name = Section::deserialize(des)?;
        let machine_name = Section {
            size: des.read_u32_le(),
            offset: machine_name_address,
        };
        let resin_density = des.read_f32_le();

        Ok(Self {
            resin_color: Vector4::new(color_b, color_g, color_r, color_a),
            machine_name: read_string(des, machine_name),
            resin_type: read_string(des, resin_type),
            resin_name: read_string(des, resin_name),
            resin_density,
        })
    }
}
