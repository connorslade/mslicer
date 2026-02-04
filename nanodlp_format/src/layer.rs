use std::mem;

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
            * (config.platform_size.x / config.platform_resolution.x as f32)
            * (config.platform_size.y / config.platform_resolution.y as f32);

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

struct RunChunks<'a> {
    runs: &'a [Run],
    width: u64,

    index: usize,
    offset: u64,
}

impl<'a> RunChunks<'a> {
    pub fn new(runs: &'a [Run], width: u32) -> Self {
        Self {
            runs,
            width: width as u64,

            index: 0,
            offset: 0,
        }
    }
}

impl<'a> Iterator for RunChunks<'a> {
    type Item = Vec<Run>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.runs.len() {
            return None;
        }

        let mut out = Vec::new();
        let mut length = 0;

        while length < self.width && self.index < self.runs.len() {
            let run = self.runs[self.index];
            let run_length = run.length - self.offset;
            let clamped_run_length = run_length.min(self.width - length);
            length += clamped_run_length;
            out.push(Run {
                length: clamped_run_length,
                value: run.value,
            });

            if clamped_run_length == run_length {
                self.index += 1;
                self.offset = 0;
            } else {
                self.offset += clamped_run_length;
            }
        }

        Some(out)
    }
}

/// Returns a list of lengths, starting with zero and alternating. So `[0, 23,
/// 7]` would mean the run starts with 23 non-zero bytes, then 7 zero bytes.
fn condense_nonzero_runs(runs: &[Run]) -> Vec<u64> {
    let mut out = Vec::new();

    let mut value = false;
    let mut length = 0;
    for run in runs {
        let this_value = run.value > 0;
        if this_value ^ value {
            out.push(mem::replace(&mut length, run.length));
            value = this_value;
        } else {
            length += run.length;
        }
    }

    (length > 0).then(|| out.push(length));
    out
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
