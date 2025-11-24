use std::fmt::Debug;

use anyhow::{Result, ensure};

use common::serde::{Deserializer, Serializer};
use sha2::{Digest, Sha256};

use crate::{Section, crypto::encrypt, layer::LayerRef, settings::Settings};

pub struct File {
    pub version: u32,
    pub settings: Settings,
    pub layers: Vec<LayerRef>,
}

impl File {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        assert_eq!(des.read_u32_le(), 0x12FD0107);

        let settings = Section::deserialize_rev(des)?;
        let settings = des.execute_at(settings.offset as usize, |des| {
            Settings::deserialize(des, settings.size as usize)
        })?;

        des.advance_by(4);
        let version = des.read_u32_le();
        let signature = Section::deserialize_rev(des)?;

        let hash = Sha256::digest(settings.checksum_value.to_le_bytes());
        let signature = des.execute_at(signature.offset as usize, |des| {
            des.read_bytes(signature.size as usize)
        });
        ensure!(encrypt(&hash) == signature);

        des.jump_to(settings.layer_pointers_offset as usize);
        let layers = (0..settings.layer_count)
            .map(|_| LayerRef::deserialize(des))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            version,
            settings,
            layers,
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(0x12FD0107);
        let settings = ser.reserve(8);
        ser.write_u32_le(0);
        ser.write_u32_le(self.version);
        let signature = ser.reserve(8);
        ser.write_u32_le(0);
        ser.write_u16_le(1);
        ser.write_u16_le(1);
        ser.write_u32_le(0);
        ser.write_u32_le(0x2A);
        ser.write_u32_le(0);

        let pos = ser.pos();
        let size = self.settings.serialize(ser);
        ser.execute_at(settings, |ser| Section::new(pos, size).serialize_rev(ser));

        let hash = Sha256::digest(self.settings.checksum_value.to_le_bytes());
        let bytes = encrypt(&hash);

        let pos = ser.pos();
        ser.write_bytes(&bytes);
        ser.execute_at(signature, |ser| {
            Section::new(pos, bytes.len()).serialize_rev(ser);
        });
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("version", &self.version)
            .field("settings", &self.settings)
            .finish()
    }
}
