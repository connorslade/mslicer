use std::fmt::Debug;

use anyhow::{Ok, Result, ensure};
use common::{
    serde::{Deserializer, SliceDeserializer},
    units::{Milimeters, Seconds},
};

use crate::shared::xor_cypher;

pub struct Layer {
    pub position_z: Milimeters,
    pub exposure_time: Seconds,
    pub light_off_delay: Seconds,
    pub data: Vec<u8>,
}

impl Layer {
    pub fn deserialize(des: &mut SliceDeserializer, key: u32, layer: u32) -> Result<Self> {
        Ok(Self {
            position_z: Milimeters::new(des.read_f32_le()),
            exposure_time: Seconds::new(des.read_f32_le()),
            light_off_delay: Seconds::new(des.read_f32_le()),
            data: {
                let address = des.read_u32_le() as usize;
                let size = des.read_u32_le() as usize;
                let _page = des.read_u32_le(); // not sure what this is...
                ensure!(des.read_u32_le() == 0x54);

                des.advance_by(8);
                let mut data = des.execute_at(address, |des| des.read_bytes(size).into_owned());
                if key != 0 {
                    xor_cypher(&mut data, key, layer);
                }

                data
            },
        })
    }
}

impl Debug for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layer")
            .field("position_z", &self.position_z)
            .field("exposure_time", &self.exposure_time)
            .field("light_off_delay", &self.light_off_delay)
            .finish()
    }
}
