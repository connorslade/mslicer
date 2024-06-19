use anyhow::{ensure, Result};

use common::serde::{Deserializer, Serializer};

use crate::{HeaderInfo, LayerContent, ENDING_STRING};

pub struct File {
    pub header: HeaderInfo,
    pub layers: Vec<LayerContent>,
}

impl File {
    pub fn new(header: HeaderInfo, layers: Vec<LayerContent>) -> Self {
        Self { header, layers }
    }
}

impl File {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.header.serialize(ser);
        for layer in &self.layers {
            layer.serialize(ser);
        }
        ser.write_bytes(ENDING_STRING);
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        let mut des = Deserializer::new(buf);

        let header = HeaderInfo::deserialize(&mut des)?;
        let mut layers = Vec::with_capacity(header.layer_count as usize);

        for _ in 0..header.layer_count {
            layers.push(LayerContent::deserialize(&mut des)?);
        }

        ensure!(des.read_bytes(ENDING_STRING.len()) == ENDING_STRING);
        Ok(Self { header, layers })
    }
}
