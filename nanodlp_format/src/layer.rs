use std::io::Cursor;

use common::{config::SliceConfig, misc::EncodableLayer};
use image::{ExtendedColorType, GrayImage, ImageEncoder, RgbImage, codecs::png::PngEncoder};
use nalgebra::Vector2;

use crate::{decode_png, types::LayerInfo};

pub struct Layer {
    pub inner: Vec<u8>,
    pub info: LayerInfo,
}

pub struct LayerEncoder {
    image: RgbImage,
    index: usize,
}

pub struct LayerDecoder {
    image: RgbImage,
}

impl LayerEncoder {
    pub fn from_gray_image(image: GrayImage) -> Self {
        let (width, height) = (image.width(), image.height());
        let image = RgbImage::from_raw(width / 3, height, image.into_raw()).unwrap();
        let index = (width * height) as usize;
        Self { image, index }
    }

    pub fn image_data(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let writer = Cursor::new(&mut bytes);
        let encoder = PngEncoder::new(writer);

        let (width, height) = (self.image.width(), self.image.height());
        encoder
            .write_image(&self.image, width, height, ExtendedColorType::Rgb8)
            .unwrap();

        bytes
    }
}

impl EncodableLayer for LayerEncoder {
    type Output = Layer;

    fn new(platform: Vector2<u32>) -> Self {
        Self {
            image: RgbImage::new(platform.x / 3, platform.y),
            index: 0,
        }
    }

    fn add_run(&mut self, length: u64, value: u8) {
        (*self.image)[self.index..(self.index + length as usize)].fill(value);
        self.index += length as usize;
    }

    fn finish(self, _layer: u64, _config: &SliceConfig) -> Self::Output {
        Layer {
            inner: self.image_data(),
            info: LayerInfo {
                total_solid_area: 0.0,
                largest_area: 0.0,
                smallest_area: 0.0,
                min_x: 0,
                min_y: 0,
                max_x: 0,
                max_y: 0,
                area_count: 0,
            },
        }
    }
}

impl LayerDecoder {
    pub fn new(data: &[u8]) -> Self {
        Self {
            image: decode_png(data).unwrap().to_rgb8(),
        }
    }

    pub fn into_inner(self) -> RgbImage {
        self.image
    }
}
