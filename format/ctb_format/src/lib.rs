//! ChituBox encrypted format v5 (`.ctb`).
//!
//! ## References
//!
//! This implementation would not be possible without the work done by the UV Tools contributors. Thank you!!!
//!
//! - [UV Tools](https://github.com/sn4k3/UVtools)
//!
//! ## Examples
//!
//! Decode a CTB file from disk and saves all of its layers as PNGs.
//!
//! ```ignore,msla_format
//! use std::fs;
//! use msla_format::{
//!     container::rle::png::{ColorType, PngEncoder},
//!     ctb,
//!     serde::{DynamicSerializer, SliceDeserializer},
//! };
//!
//! let bytes = fs::read("out.ctb").unwrap();
//! let mut des = SliceDeserializer::new(&bytes);
//! let file = ctb::File::deserialize(&mut des).unwrap();
//! println!("{file:?}");
//!
//! for (i, layer) in file.layers.iter().enumerate() {
//!     let decoder = ctb::LayerDecoder::new(&layer.data);
//!
//!     let mut ser = DynamicSerializer::new();
//!     let mut png = PngEncoder::new(&mut ser, ColorType::Grayscale, file.resolution);
//!     png.write_image_data(decoder.collect());
//!     png.write_end();
//!
//!     fs::write(format!("layer_{i}.png"), ser.into_inner()).unwrap();
//! }
//! ```

use anyhow::Result;

use common::serde::{Deserializer, Serializer, SliceDeserializer};

mod crypto;
mod file;
mod layer;
mod layer_coding;
mod preview;
mod resin;

pub use crate::{
    file::File,
    layer::Layer,
    layer_coding::{LayerDecoder, LayerEncoder},
    preview::PreviewImage,
    resin::ResinParameters,
};

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
