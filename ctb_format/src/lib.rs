use aes::{
    Aes256,
    cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::NoPadding},
};
use anyhow::Result;

use common::serde::Deserializer;

pub mod header;
pub mod settings;

// thank u uv tools :pray:
const ENCRYPT_KEY: &[u8; 32] = &[
    0xD0, 0x5B, 0x8E, 0x33, 0x71, 0xDE, 0x3D, 0x1A, 0xE5, 0x4F, 0x22, 0xDD, 0xDF, 0x5B, 0xFD, 0x94,
    0xAB, 0x5D, 0x64, 0x3A, 0x9D, 0x7E, 0xBF, 0xAF, 0x42, 0x03, 0xF3, 0x10, 0xD8, 0x52, 0x2A, 0xEA,
];
const ENCRYPT_IV: &[u8; 16] = &[
    0x0F, 0x01, 0x0A, 0x05, 0x05, 0x0B, 0x06, 0x07, 0x08, 0x06, 0x0A, 0x0C, 0x0C, 0x0D, 0x09, 0x0F,
];

#[derive(Debug)]
pub struct Section {
    pub size: u32,
    pub offset: u32,
}

impl Section {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            size: des.read_u32_le(),
            offset: des.read_u32_le(),
        })
    }

    pub fn deserialize_rev(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            offset: des.read_u32_le(),
            size: des.read_u32_le(),
        })
    }
}

pub fn decrypt(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    cbc::Decryptor::<Aes256>::new(ENCRYPT_KEY.into(), ENCRYPT_IV.into())
        .decrypt_padded_mut::<NoPadding>(&mut bytes)
        .unwrap();
    bytes
}

pub fn encrypt(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    let length = bytes.len();

    cbc::Encryptor::<Aes256>::new(ENCRYPT_KEY.into(), ENCRYPT_IV.into())
        .encrypt_padded_mut::<NoPadding>(&mut bytes, length)
        .unwrap();

    bytes
}
