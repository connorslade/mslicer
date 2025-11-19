use std::fs;

use anyhow::{Ok, Result};

use common::serde::Deserializer;
use ctb_format::{decrypt, encrypt, header::Header, settings::Settings};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let file = fs::read("output-enc-5.ctb")?;
    let mut des = Deserializer::new(&file);

    let header = Header::deserialize(&mut des)?;
    assert_eq!(header.magic, 0x12FD0107);
    dbg!(&header);

    des.jump_to(header.settings.offset as usize);
    let bytes = decrypt(des.read_bytes(header.settings.size as usize));
    let settings = Settings::deserialize(&mut Deserializer::new(&bytes))?;
    dbg!(&settings);

    let hash = Sha256::digest(settings.checksum_value.to_le_bytes());
    let encrypted = encrypt(&hash);

    des.jump_to(header.signature.offset as usize);
    let signature = des.read_bytes(header.signature.size as usize);
    assert_eq!(encrypted, signature);

    Ok(())
}
