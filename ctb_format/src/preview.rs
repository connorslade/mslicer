use std::{
    fmt::{self, Debug},
    iter::repeat_n,
};

use anyhow::Result;

use common::serde::Deserializer;
use nalgebra::{Vector2, Vector3};

use crate::Section;

pub struct PreviewImage {
    width: usize,
    data: Vec<Vector3<u8>>,
}

struct Preview {
    width: u32,
    height: u32,
    section: Section,
}

impl PreviewImage {
    pub fn deserialize(des: &mut Deserializer) -> Result<Self> {
        let preview = Preview::deserialize(des)?;
        des.jump_to(preview.section.offset as usize);
        PreviewImage::from_bytes(
            des.read_bytes(preview.section.size as usize),
            Vector2::new(preview.width, preview.height),
        )
    }

    pub fn from_bytes(bytes: &[u8], size: Vector2<u32>) -> Result<Self> {
        let pixels = size.x as usize * size.y as usize;
        let mut data = Vec::with_capacity(pixels);

        let mut i = 0;
        while i < bytes.len() {
            let run = u16::from_le_bytes([bytes[i], bytes[i + 1]]);
            i += 2;

            let red = (((run >> 11) & 0x1F) << 3) as u8;
            let green = (((run >> 6) & 0x1F) << 3) as u8;
            let blue = ((run & 0x1F) << 3) as u8;

            let mut count = 1;
            if run & 0x20 != 0 {
                count += (bytes[i + 1] as u16 & 0x0F) << 8 | bytes[i] as u16;
                i += 2;
            }

            data.extend(repeat_n(Vector3::new(red, green, blue), count as usize));
        }

        if data.len() < pixels {
            data.resize(pixels, Vector3::zeros());
        }

        Ok(Self {
            width: size.x as usize,
            data,
        })
    }

    pub fn size(&self) -> Vector2<u32> {
        Vector2::new(self.width as u32, (self.data.len() / self.width) as u32)
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Vector3<u8> {
        let index = y * self.width + x;
        self.data[index]
    }
}

impl Preview {
    fn deserialize(des: &mut Deserializer) -> Result<Self> {
        Ok(Self {
            width: des.read_u32_le(),
            height: des.read_u32_le(),
            section: Section::deserialize(des)?,
        })
    }
}

impl Debug for PreviewImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size();
        f.debug_struct("PreviewImage")
            .field("width", &self.width)
            .field("height", &size.y)
            .finish()
    }
}
