use std::io::{Cursor, Read};

use anyhow::{Ok, Result};
use image::{DynamicImage, ImageFormat, codecs::png::PngDecoder};

mod file;
mod layer;
pub mod png;
mod types;
pub use crate::{
    file::File,
    layer::{Layer, LayerDecoder, LayerEncoder},
};

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

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    Ok(bytes)
}
