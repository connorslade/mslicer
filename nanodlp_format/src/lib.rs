use std::io::{Cursor, Read};

use anyhow::{Ok, Result};
use image::{DynamicImage, codecs::png::PngDecoder};

pub mod file;
mod types;

fn read_to_bytes<T: Read>(mut reader: T) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

fn decode_png(png: &[u8]) -> Result<DynamicImage> {
    let decoder = PngDecoder::new(Cursor::new(png))?;
    let image = DynamicImage::from_decoder(decoder)?;
    Ok(image)
}
