//! ChituBox encrypted format v5 (`.ctb`).
//!
//! ## References
//!
//! This implementation would not be possible without the work done by the UV Tools contributors. Thank you!!!
//!
//! - [UV Tools](https://github.com/sn4k3/UVtools)

use aes::{
    Aes256,
    cipher::{
        BlockDecryptMut, BlockEncryptMut, KeyIvInit,
        block_padding::{NoPadding, ZeroPadding},
    },
};
use anyhow::Result;

use common::serde::{Deserializer, Serializer, SliceDeserializer};

mod encoding;
mod file;
mod layer;
mod preview;
mod resin;

pub use crate::{
    encoding::{LayerDecoder, LayerEncoder},
    file::File,
    layer::Layer,
    preview::PreviewImage,
    resin::ResinParameters,
};

// thank u uv tools :pray:
const ENCRYPT_KEY: &[u8; 32] = &[
    0xD0, 0x5B, 0x8E, 0x33, 0x71, 0xDE, 0x3D, 0x1A, 0xE5, 0x4F, 0x22, 0xDD, 0xDF, 0x5B, 0xFD, 0x94,
    0xAB, 0x5D, 0x64, 0x3A, 0x9D, 0x7E, 0xBF, 0xAF, 0x42, 0x03, 0xF3, 0x10, 0xD8, 0x52, 0x2A, 0xEA,
];
const ENCRYPT_IV: &[u8; 16] = &[
    0x0F, 0x01, 0x0A, 0x05, 0x05, 0x0B, 0x06, 0x07, 0x08, 0x06, 0x0A, 0x0C, 0x0C, 0x0D, 0x09, 0x0F,
];

#[derive(Debug)]
struct Section {
    pub size: u32,
    pub offset: u32,
}

impl Section {
    pub fn new(offset: usize, size: usize) -> Self {
        Self {
            size: size as u32,
            offset: offset as u32,
        }
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            offset: des.read_u32_le(),
            size: des.read_u32_le(),
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(self.offset);
        ser.write_u32_le(self.size);
    }

    pub fn deserialize_rev(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            size: des.read_u32_le(),
            offset: des.read_u32_le(),
        })
    }

    pub fn serialize_rev<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_le(self.size);
        ser.write_u32_le(self.offset);
    }
}

fn read_string(des: &mut SliceDeserializer, section: Section) -> String {
    des.execute_at(section.offset as usize, |des| {
        String::from_utf8_lossy(des.read_slice(section.size as usize)).into_owned()
    })
}

pub fn decrypt(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    decrypt_in_place(&mut bytes);
    bytes
}

pub fn decrypt_in_place(bytes: &mut [u8]) {
    let out = cbc::Decryptor::<Aes256>::new(ENCRYPT_KEY.into(), ENCRYPT_IV.into())
        .decrypt_padded_mut::<NoPadding>(bytes)
        .unwrap();
    assert_eq!(out.len(), bytes.len());
}

pub fn encrypt(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    encrypt_in_place(&mut bytes);
    bytes
}

pub fn encrypt_in_place(bytes: &mut Vec<u8>) {
    bytes.resize(bytes.len().next_multiple_of(32), 0);
    let length = bytes.len();

    cbc::Encryptor::<Aes256>::new(ENCRYPT_KEY.into(), ENCRYPT_IV.into())
        .encrypt_padded_mut::<ZeroPadding>(bytes, length)
        .unwrap();
}
