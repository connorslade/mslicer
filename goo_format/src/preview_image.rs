use common::serde::{Deserializer, Serializer};

pub struct PreviewImage<const WIDTH: usize, const HEIGHT: usize> {
    // 0brrrrrggggggbbbbb
    data: [[u16; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> PreviewImage<WIDTH, HEIGHT> {
    pub const fn empty() -> Self {
        Self {
            data: [[0; WIDTH]; HEIGHT],
        }
    }

    pub fn serializes<T: Serializer>(&self, serializer: &mut T) {
        for row in self.data.iter() {
            for pixel in row.iter() {
                serializer.write_u16(*pixel);
            }
        }
    }

    pub fn deserializes(deserializer: &mut Deserializer) -> Self {
        let mut data = [[0; WIDTH]; HEIGHT];
        for row in data.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = deserializer.read_u16();
            }
        }
        Self { data }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (f32, f32, f32)) {
        let red = (color.0 * 31.0).round() as u16;
        let green = (color.1 * 63.0).round() as u16;
        let blue = (color.2 * 31.0).round() as u16;
        self.data[y][x] = (red << 11) | (green << 5) | blue;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (f32, f32, f32) {
        let pixel = self.data[y][x];
        let red = ((pixel >> 11) & 0x1F) as f32 / 31.0;
        let green = ((pixel >> 5) & 0x3F) as f32 / 63.0;
        let blue = (pixel & 0x1F) as f32 / 31.0;
        (red, green, blue)
    }
}
