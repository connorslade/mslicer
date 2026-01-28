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
    platform: Vector2<u32>,
    runs: Vec<Run>,
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

        let mut out = LayerEncoder::new(platform);
        (image.runs()).for_each(|run| out.add_run(run.length, run.value));
        out
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
            platform,
            runs: Vec::new(),
        }
    }

    fn add_run(&mut self, length: u64, value: u8) {
        self.runs.push(Run { length, value });
    }

    fn finish(self, _layer: u64, _config: &SliceConfig) -> Self::Output {
        let mut area = 0;
        let mut pos = 0;

        let mut min = Vector2::repeat(u64::MAX);
        let mut max = Vector2::repeat(u64::MIN);

        let width = self.platform.x as u64;
        for run in self.runs.iter() {
            if run.value > 0 {
                let y = pos / width;
                area += run.length;

                min.x = min.x.min(pos % width);
                min.y = min.y.min(y);
                pos += run.length;
                max.x = max.x.max(pos % width);
                max.y = max.y.max(y);
            } else {
                pos += run.length;
            }
        }

        Layer {
            info: LayerInfo {
                total_solid_area: area as f32,
                largest_area: area as f32,  // todo
                smallest_area: area as f32, // todo
                min_x: min.x as u32,
                min_y: min.y as u32,
                max_x: max.x as u32,
                max_y: max.y as u32,
                area_count: 1,
            },
            inner: self.image_data(),
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
