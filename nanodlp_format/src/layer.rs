use common::{
    config::SliceConfig,
    image::Image,
    misc::{EncodableLayer, Run},
    serde::DynamicSerializer,
};
use image::{GrayImage, RgbImage};
use nalgebra::Vector2;

use crate::{
    decode_png,
    png::{PngEncoder, PngInfo},
    types::LayerInfo,
};

pub struct Layer {
    pub inner: Vec<u8>,
    pub info: LayerInfo,
}

pub struct LayerEncoder {
    runs: Vec<Run>,
    platform: Vector2<u32>,
}

pub struct LayerDecoder {
    image: RgbImage,
}

impl LayerEncoder {
    pub fn from_gray_image(gray_image: GrayImage) -> Self {
        let platform = Vector2::new(gray_image.width(), gray_image.height());
        let image = Image::from_raw(
            gray_image.width() as usize,
            gray_image.height() as usize,
            gray_image.into_raw(),
        );

        Self {
            platform,
            runs: image.runs().collect(),
        }
    }

    pub fn image_data(self) -> Vec<u8> {
        let mut ser = DynamicSerializer::new();

        let info = PngInfo {
            width: self.platform.x / 3,
            height: self.platform.y,
            bit_depth: 8,
            color_type: 2,
        };

        let mut encoder = PngEncoder::new(&mut ser, &info, 3);
        encoder.write_pixel_dimensions(3, 1);
        encoder.write_image_data(self.runs);
        encoder.write_end();
        ser.into_inner()
    }
}

impl EncodableLayer for LayerEncoder {
    type Output = Layer;

    fn new(platform: Vector2<u32>) -> Self {
        Self {
            runs: Vec::new(),
            platform,
        }
    }

    fn add_run(&mut self, length: u64, value: u8) {
        self.runs.push(Run { length, value });
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
