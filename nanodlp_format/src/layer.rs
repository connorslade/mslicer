use common::{
    container::{
        Image,
        rle::{
            Run, RunChunks, condense_nonzero_runs,
            png::{ColorType, PngEncoder},
        },
    },
    serde::DynamicSerializer,
    slice::{EncodableLayer, SliceConfig},
    units::Milimeter,
};
use image::{GrayImage, RgbImage};
use nalgebra::Vector2;

use crate::{decode_png, types::LayerInfo};

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

        let resolution = Vector2::new(self.platform.x.div_ceil(3), self.platform.y);
        let mut encoder = PngEncoder::new(&mut ser, ColorType::Truecolor, resolution);
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

    fn finish(self, _layer: u64, config: &SliceConfig) -> Self::Output {
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

        let chunks = RunChunks::new(&self.runs, config.platform_resolution.x);
        let mut prev = vec![];
        let mut area_count = 0i32;

        for row in chunks {
            let condensed = condense_nonzero_runs(&row);
            if prev.is_empty() {
                area_count += (condensed.len() / 2) as i32;
                prev = condensed;
                continue;
            }

            area_count += unmatched_runs(&prev, &condensed);
            prev = condensed;
        }

        let area = area as f32
            * (config.platform_size.x.get::<Milimeter>() / config.platform_resolution.x as f32)
            * (config.platform_size.y.get::<Milimeter>() / config.platform_resolution.y as f32);

        let area_count = area_count.max(1);
        Layer {
            info: LayerInfo {
                // todo: correctly set largest and smallest area
                total_solid_area: area,
                largest_area: area,
                smallest_area: area / area_count as f32,
                min_x: min.x as u32,
                min_y: min.y as u32,
                max_x: max.x as u32,
                max_y: max.y as u32,
                area_count: area_count as u32,
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

fn unmatched_runs(prev: &[u64], next: &[u64]) -> i32 {
    let mut next_pos = 0;
    let mut next_idx = 0;
    let mut sum = 0;

    while next_idx < next.len() {
        let next_len = next[next_idx];
        if next_idx % 2 == 1 {
            let next_end = next_pos + next_len;
            let mut touched_count = 0;
            let mut prev_pos = 0;

            for (prev_idx, &prev_run_length) in prev.iter().enumerate() {
                touched_count += (prev_idx % 2 == 1
                    && next_pos < prev_pos + prev_run_length
                    && prev_pos < next_end) as i32;
                prev_pos += prev_run_length;

                if prev_pos >= next_end {
                    break;
                }
            }

            sum += 1 - touched_count;
        }

        next_pos += next_len;
        next_idx += 1;
    }

    sum
}
