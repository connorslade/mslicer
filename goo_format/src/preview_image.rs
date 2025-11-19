use std::fmt::Debug;

use common::serde::{Deserializer, Serializer};
use image::{imageops::FilterType, RgbaImage};

pub struct PreviewImage<const WIDTH: usize, const HEIGHT: usize> {
    // 0brrrrrggggggbbbbb
    data: Box<[u16]>,
}

impl<const WIDTH: usize, const HEIGHT: usize> PreviewImage<WIDTH, HEIGHT> {
    pub fn empty() -> Self {
        let data = vec![0; WIDTH * HEIGHT];

        Self {
            data: data.into_boxed_slice(),
        }
    }

    pub fn from_image(image: &RgbaImage) -> Self {
        assert_eq!(image.width() as usize, WIDTH);
        assert_eq!(image.height() as usize, HEIGHT);

        let mut data = vec![0; WIDTH * HEIGHT];

        for (i, pixel) in data.iter_mut().enumerate() {
            let (x, y) = (i % WIDTH, i / WIDTH);
            let color = image.get_pixel(x as u32, y as u32);

            let red = (color[0] as u16) >> 3;
            let green = (color[1] as u16) >> 2;
            let blue = (color[2] as u16) >> 3;

            *pixel = (red << 11) | (green << 5) | blue;
        }

        Self {
            data: data.into_boxed_slice(),
        }
    }

    pub fn from_image_scaled(image: &RgbaImage, filter: FilterType) -> Self {
        let scaled = image::imageops::resize(image, WIDTH as u32, HEIGHT as u32, filter);
        Self::from_image(&scaled)
    }

    pub fn inner_data(&self) -> &[u16] {
        &self.data
    }

    pub fn serializes<T: Serializer>(&self, serializer: &mut T) {
        for pixel in self.data.iter() {
            serializer.write_u16_be(*pixel);
        }
    }

    pub fn deserializes(deserializer: &mut Deserializer) -> Self {
        let mut out = Self::empty();

        for pixel in out.data.iter_mut() {
            *pixel = deserializer.read_u16_be();
        }

        out
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (f32, f32, f32)) {
        let red = (color.0 * 31.0).round() as u16;
        let green = (color.1 * 63.0).round() as u16;
        let blue = (color.2 * 31.0).round() as u16;
        self.data[y * WIDTH + x] = (red << 11) | (green << 5) | blue;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (f32, f32, f32) {
        let pixel = self.data[y * WIDTH + x];
        let red = ((pixel >> 11) & 0x1F) as f32 / 31.0;
        let green = ((pixel >> 5) & 0x3F) as f32 / 63.0;
        let blue = (pixel & 0x1F) as f32 / 31.0;
        (red, green, blue)
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Debug for PreviewImage<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewImage").finish()
    }
}
