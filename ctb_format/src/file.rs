use std::fmt::Debug;

use anyhow::{Result, ensure};

use common::serde::Deserializer;
use sha2::{Digest, Sha256};

use crate::{
    Section, crypto::encrypt, layer::LayerRef, resin::ResinParameters, settings::Settings,
};

pub struct File {
    pub version: u32,
    pub settings: Settings,
    pub resin: ResinParameters,
    pub layers: Vec<LayerRef>,
}

impl File {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        assert_eq!(des.read_u32_le(), 0x12FD0107);

        let settings = Section::deserialize(des)?;
        let settings = des.execute_at(settings.offset as usize, |des| {
            Settings::deserialize(des, settings.size as usize)
        })?;

        des.advance_by(4);
        let version = des.read_u32_le();
        let signature = Section::deserialize(des)?;

        let hash = Sha256::digest(settings.checksum_value.to_le_bytes());
        let signature = des.execute_at(signature.offset as usize, |des| {
            des.read_bytes(signature.size as usize)
        });
        ensure!(encrypt(&hash) == signature);

        let resin = des.execute_at(settings.resin_parameters_address as usize, |des| {
            ResinParameters::deserialize(des)
        })?;

        des.jump_to(settings.layer_pointers_offset as usize);
        let layers = (0..settings.layer_count)
            .map(|_| LayerRef::deserialize(des))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            version,
            settings,
            resin,
            layers,
        })
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("version", &self.version)
            .field("settings", &self.settings)
            .field("resin", &self.resin)
            .finish()
    }
}
