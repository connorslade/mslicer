use std::{
    fmt::{self, Debug},
    iter::repeat_n,
    mem,
};

use anyhow::Result;

use common::serde::{Deserializer, Serializer, SliceDeserializer};
use image::RgbaImage;
use nalgebra::{Vector2, Vector3};

use crate::Section;

#[derive(Default)]
pub struct PreviewImage {
    width: usize,
    data: Vec<Vector3<u8>>,
}

impl PreviewImage {
    pub fn from_image(image: &RgbaImage) -> Self {
        let mut data = Vec::with_capacity((image.width() * image.height()) as usize);
        for color in image.chunks(4) {
            data.push(Vector3::new(color[0], color[1], color[2]));
        }

        Self {
            width: image.width() as usize,
            data,
        }
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        let width = des.read_u32_le();
        let height = des.read_u32_le();
        let image = Section::deserialize(des)?;

        des.jump_to(image.offset as usize);
        PreviewImage::from_bytes(
            des.read_slice(image.size as usize),
            Vector2::new(width, height),
        )
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        let size = self.size();
        ser.write_u32_le(size.x);
        ser.write_u32_le(size.y);
        let section = ser.reserve(8);

        let data = self.to_bytes();
        let offset = ser.pos();
        ser.write_bytes(&data);
        ser.execute_at(section, |ser| {
            Section::new(offset, data.len()).serialize(ser)
        });
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut last_color = self.data[0];
        let mut length = 0;

        let mut add_run = |length: u32, color: Vector3<u8>| {
            let color = ((color.z >> 3) as u16)
                | ((color.y >> 2) as u16) << 5
                | ((color.x >> 3) as u16) << 11;

            match length {
                0 => {}
                x @ 1..=2 => {
                    let value = color & !0x20;
                    out.extend(repeat_n([value as u8, (value >> 8) as u8], x as usize).flatten());
                }
                x => {
                    let value = color | 0x20;
                    out.extend([value as u8, (value >> 8) as u8]);
                    let value = (x - 1) | 0x3000;
                    out.extend([value as u8, (value >> 8) as u8]);
                }
            }
        };

        for pixel in self.data.iter() {
            if *pixel == last_color {
                length += 1;
                if length == 0xFFF {
                    add_run(length, last_color);
                    length = 0;
                }
            } else {
                add_run(length, last_color);
                last_color = *pixel;
                length = 1;
            }
        }

        add_run(length, last_color);

        out
    }

    pub fn inner_data(&self) -> &[u8] {
        unsafe { mem::transmute(self.data.as_slice()) }
    }

    pub fn size(&self) -> Vector2<u32> {
        Vector2::new(self.width as u32, (self.data.len() / self.width) as u32)
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Vector3<u8> {
        let index = y * self.width + x;
        self.data[index]
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
